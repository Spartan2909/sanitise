#![feature(lint_reasons, proc_macro_expand)]
#![doc = include_str!("../README.md")]
#![deny(clippy::todo, clippy::unwrap_used)]

mod output;
use output::parse_output;
mod to_tokens;

use std::{
    collections::{HashMap, VecDeque},
    fmt,
    iter::zip,
};

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, LitStr, Token,
};
use yaml_rust::{yaml::Hash, Yaml, YamlLoader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnType {
    Bool,
    Float,
    Integer,
    String,
}

impl ColumnType {
    fn is_numeric(&self) -> bool {
        match self {
            ColumnType::Float | ColumnType::Integer => true,
            ColumnType::Bool | ColumnType::String => false,
        }
    }
}

impl From<&str> for ColumnType {
    fn from(value: &str) -> Self {
        match value {
            "boolean" => ColumnType::Bool,
            "float" => ColumnType::Float,
            "integer" => ColumnType::Integer,
            "string" => ColumnType::String,
            _ => panic!("invalid column type {value}"),
        }
    }
}

impl From<&Value> for ColumnType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Boolean(_) => ColumnType::Bool,
            Value::Integer(_) => ColumnType::Integer,
            Value::Real(_) => ColumnType::Float,
            Value::String(_) => ColumnType::String,
        }
    }
}

impl From<Value> for ColumnType {
    fn from(value: Value) -> Self {
        (&value).into()
    }
}

impl From<String> for ColumnType {
    fn from(value: String) -> Self {
        ColumnType::from(value.as_str())
    }
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnType::Bool => write!(f, "boolean"),
            ColumnType::Float => write!(f, "real"),
            ColumnType::Integer => write!(f, "integer"),
            ColumnType::String => write!(f, "string"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
}

impl From<&Yaml> for Value {
    fn from(value: &Yaml) -> Self {
        match value {
            Yaml::Integer(i) => Value::Integer(*i),
            Yaml::Real(s) => Value::Real(
                s.parse()
                    .unwrap_or_else(|_| panic!("invalid real {:#?}", value)),
            ),
            Yaml::String(s) => Value::String(s.to_owned()),
            _ => panic!("invalid value {:#?}", value),
        }
    }
}

impl From<Yaml> for Value {
    fn from(value: Yaml) -> Self {
        Value::from(&value)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum OnInvalid {
    Abort,
    Average(usize),
    Delete,
    Previous(Value),
    Sentinel(Value),
}

fn get_on_invalid(yaml: &Yaml, hash: &mut Hash, kind: &str) -> OnInvalid {
    let on_invalid = yaml
        .as_str()
        .unwrap_or_else(|| panic!("value of on-{kind} must be a string"));

    match on_invalid {
        "abort" => OnInvalid::Abort,
        "average" => {
            let valid_streak = hash
                .remove(&Yaml::from_str("valid-streak"))
                .unwrap_or_else(|| {
                    panic!("'average' option for on-{kind} requires key 'valid-streak'")
                })
                .into_i64()
                .and_then(|n| n.try_into().ok())
                .expect("'valid-streak' must be a positive integer");
            OnInvalid::Average(valid_streak)
        }
        "delete" => OnInvalid::Delete,
        "previous" => {
            let key = format!("{kind}-sentinel");
            let sentinel = hash
                .remove(&Yaml::from_str(&key))
                .unwrap_or_else(|| {
                    panic!("'previous' option for on-{kind} requires key '{kind}-sentinel'")
                })
                .into();
            OnInvalid::Previous(sentinel)
        }
        "sentinel" => {
            let key = format!("{kind}-sentinel");
            let sentinel = hash
                .remove(&Yaml::from_str(&key))
                .unwrap_or_else(|| {
                    panic!("'sentinel' option for on-{kind} requires key '{kind}-sentinel'")
                })
                .into();
            OnInvalid::Sentinel(sentinel)
        }
        _ => panic!("invalid value for on-{kind}: '{on_invalid}'"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
        };

        write!(f, "{string}")
    }
}

impl BinOp {
    fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge
        )
    }

    fn is_numeric(&self) -> bool {
        matches!(
            self,
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnOp {
    Negate,
    Not,
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            UnOp::Negate => "-",
            UnOp::Not => "!",
        };

        write!(f, "{string}")
    }
}

#[derive(Debug, Clone)]
enum Function {
    Boolean(Box<Output>),
    Ceiling(Box<Output>),
    Concat(Box<Output>, Box<Output>),
    Floor(Box<Output>),
    Integer(Box<Output>),
    Real(Box<Output>),
    Round(Box<Output>),
    String(Box<Output>),
}

impl Function {
    /// Gets the type that this expression will evaluate to.
    /// # Errors
    /// Returns an error if a type error is encountered.
    fn return_type(&self, var_types: &HashMap<Ident, ColumnType>) -> Result<ColumnType, String> {
        match self {
            Function::Boolean(_) => Ok(ColumnType::Bool),
            Function::Integer(_) => Ok(ColumnType::Integer),
            Function::Real(_) => Ok(ColumnType::Float),
            Function::String(_) => Ok(ColumnType::String),
            Function::Ceiling(output) | Function::Floor(output) | Function::Round(output) => {
                if output.return_type(var_types)? == ColumnType::Float {
                    Ok(ColumnType::Float)
                } else {
                    Err(format!("argument to '{self}' must be a real"))
                }
            }
            Function::Concat(left, right) => {
                let left_type = left.return_type(var_types)?;
                let right_type = right.return_type(var_types)?;
                if left_type != ColumnType::String || right_type != ColumnType::String {
                    Err("arguments to 'concat' must be strings".to_string())
                } else {
                    Ok(ColumnType::String)
                }
            }
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Function::Boolean(_) => "boolean",
            Function::Ceiling(_) => "ceiling",
            Function::Concat(_, _) => "concat",
            Function::Floor(_) => "floor",
            Function::Integer(_) => "integer",
            Function::Real(_) => "real",
            Function::Round(_) => "round",
            Function::String(_) => "string",
        };

        write!(f, "{string}")
    }
}

#[derive(Debug, Clone)]
enum Output {
    Binary {
        left: Box<Output>,
        operator: BinOp,
        right: Box<Output>,
    },
    Function(Function),
    Identifier(Ident),
    Literal(Value),
    Unary {
        operator: UnOp,
        right: Box<Output>,
    },
}

impl Output {
    /// Gets the type that this expression will evaluate to.
    /// # Errors
    /// Returns an error if a type error is encountered.
    fn return_type(&self, var_types: &HashMap<Ident, ColumnType>) -> Result<ColumnType, String> {
        match self {
            Output::Binary {
                left,
                operator,
                right,
            } => {
                let left_type = left.return_type(var_types)?;
                let right_type = right.return_type(var_types)?;

                if left_type != right_type {
                    Err(format!("cannot compare {left_type} with {right_type}"))?
                } else if operator.is_numeric() && !left_type.is_numeric() {
                    Err(format!("cannot use operator '{operator}' on {left_type}"))?
                }

                if operator.is_comparison() {
                    Ok(ColumnType::Bool)
                } else {
                    Ok(left_type)
                }
            }
            Output::Function(function) => function.return_type(var_types),
            Output::Identifier(ident) => var_types
                .get(ident)
                .map(|column_type| Ok(*column_type))
                .unwrap_or(Err(format!("identifier '{ident}' not found"))),
            Output::Literal(value) => Ok(value.into()),
            Output::Unary { operator, right } => match *operator {
                UnOp::Negate => {
                    let right_type = right.return_type(var_types)?;
                    if !right_type.is_numeric() {
                        panic!("cannot use operator '{operator}' on {right_type}");
                    }
                    Ok(right_type)
                }
                UnOp::Not => {
                    let right_type = right.return_type(var_types)?;
                    if right_type != ColumnType::Bool {
                        panic!("cannot use operator '{operator}' on {right_type}");
                    }
                    Ok(ColumnType::Bool)
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Aggregate {
    Average,
    First,
    Last,
}

impl From<&str> for Aggregate {
    fn from(value: &str) -> Self {
        match value {
            "average" => Aggregate::Average,
            "first" => Aggregate::First,
            "last" => Aggregate::Last,
            _ => panic!("invalid value for 'aggregate': '{value}'"),
        }
    }
}

#[derive(Debug, Clone)]
struct Column {
    title: String,
    column_type: ColumnType,
    output_type: ColumnType,
    null_surrogates: Option<Vec<Value>>,
    valid_values: Option<Vec<Value>>,
    on_invalid: OnInvalid,
    on_null: OnInvalid,
    max: Option<Value>,
    min: Option<Value>,
    invalid_values: Option<Vec<Value>>,
    output: Output,
    ignore: bool,
    aggregate: Aggregate,
    process_columns: Vec<(Ident, ColumnType)>,
}

impl Column {
    fn needs_state(&self) -> bool {
        matches!(self.on_invalid, OnInvalid::Average(_))
            || matches!(self.on_null, OnInvalid::Average(_))
    }
}

#[derive(Debug, Clone)]
struct Process {
    name: String,
    columns: Vec<Column>,
    aggregate_column: Option<Ident>,
}

impl Process {
    fn signiature(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for column in &self.columns {
            if column.ignore {
                continue;
            }

            let output_type = column.output_type;

            tokens.extend(quote!(Vec<#output_type>,));
        }

        quote!((#tokens))
    }

    fn column_names(&self) -> Vec<Ident> {
        self.columns
            .iter()
            .map(|column| Ident::new(&column.title, Span::call_site()))
            .collect()
    }

    fn column_types(&self) -> Vec<ColumnType> {
        self.columns
            .iter()
            .map(|column| column.column_type)
            .collect()
    }

    fn header(&self, trailing_comma: bool) -> String {
        let mut header = String::new();
        for (i, column) in self.columns.iter().enumerate() {
            header += &column.title;
            if i + 1 < self.columns.len() {
                header += ",";
            }
        }

        if trailing_comma {
            header += ",";
        }

        header
    }

    fn ignores(&self) -> Vec<bool> {
        self.columns.iter().map(|column| column.ignore).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OnTitle {
    Combine,
    Once,
    Split,
}

impl From<Yaml> for OnTitle {
    fn from(value: Yaml) -> Self {
        let value = value
            .into_string()
            .expect("value of 'on_title' must be a string");
        let on_title: &str = &value;

        match on_title {
            "combine" => OnTitle::Combine,
            "once" => OnTitle::Once,
            "split" => OnTitle::Split,
            _ => panic!("invalid value for title: '{on_title}'"),
        }
    }
}

#[derive(Debug, Clone)]
struct Program {
    processes: Vec<Process>,
    on_title: OnTitle,
    csv: Expr,
    string_input: bool,
}

fn ensure_empty(hash: &Hash, map_name: &str) {
    if let Some(key) = hash.keys().next() {
        panic!(
            "unexpected key '{}' in {}",
            key.as_str().expect("all keys should be strings"),
            map_name,
        )
    }
}

fn parse_column(input: Yaml) -> Column {
    let mut input = input.into_hash().expect("'columns' entires must be maps");

    let title = input
        .remove(&Yaml::from_str("title"))
        .expect("column title required")
        .into_string()
        .expect("column title must be a string");

    let column_type = input
        .remove(&Yaml::from_str("column-type"))
        .expect("column type required")
        .into_string()
        .expect("column type must be a string")
        .into();

    let ignore = input
        .remove(&Yaml::from_str("ignore"))
        .map(|yaml| yaml.as_bool().expect("'ignore' must be a Boolean"))
        .unwrap_or(false);

    if ignore {
        ensure_empty(&input, "column");

        return Column {
            title,
            column_type,
            output_type: column_type,
            null_surrogates: None,
            valid_values: None,
            on_invalid: OnInvalid::Abort,
            on_null: OnInvalid::Abort,
            max: None,
            min: None,
            invalid_values: None,
            output: Output::Identifier(Ident::new("value", Span::call_site())),
            ignore,
            aggregate: Aggregate::First,
            process_columns: vec![],
        };
    }

    let output_type = input
        .remove(&Yaml::from_str("output-type"))
        .map(|yaml| {
            yaml.into_string()
                .expect("column type must be a string")
                .into()
        })
        .unwrap_or(column_type);

    let null_surrogates = input.remove(&Yaml::from_str("null-surrogates")).map(|yaml| {
            yaml.into_vec()
                .expect("'null-surrogate' must be an array")
                .into_iter()
                .map(|yaml| {
                    let value: Value = yaml.into();
                    if ColumnType::from(&value) != column_type {
                        panic!("the type of the values in 'null-surrogate' must be the same as 'columm-type'")
                    }
                    value
                })
                .collect()
        });

    let valid_values = input.remove(&Yaml::from_str("valid-values")).map(|yaml| {
        yaml.into_vec()
            .expect("'valid-values' must be an array")
            .into_iter()
            .map(|yaml| {
                let value: Value = yaml.into();
                if ColumnType::from(&value) != column_type {
                    panic!("the type of the values in 'valid-values' must be the same as 'columm-type'")
                }
                value
            })
            .collect()
    });

    let on_invalid = input
        .remove(&Yaml::from_str("on-invalid"))
        .map(|yaml| get_on_invalid(&yaml, &mut input, "invalid"))
        .unwrap_or(OnInvalid::Abort);

    let on_null = input
        .remove(&Yaml::from_str("on-null"))
        .map(|yaml| get_on_invalid(&yaml, &mut input, "null"))
        .unwrap_or(OnInvalid::Abort);

    if matches!(on_null, OnInvalid::Average(_)) && !matches!(on_invalid, OnInvalid::Average(_)) {
        panic!("'on-null' can only be 'average' if 'on-invalid' is also 'average'")
    }

    let max = input.remove(&Yaml::from_str("max")).map(|yaml| {
        let value: Value = yaml.into();
        if ColumnType::from(&value) != column_type {
            panic!("the type of 'max' must be the same as 'columm-type'")
        }
        value
    });

    let min = input.remove(&Yaml::from_str("min")).map(|yaml| {
        let value: Value = yaml.into();
        if ColumnType::from(&value) != column_type {
            panic!("the type of 'min' must be the same as 'columm-type'")
        }
        value
    });

    let invalid_values = input.remove(&Yaml::from_str("invalid-values")).map(|yaml| {
        yaml.into_vec()
            .expect("'invalid-values' must be an array")
            .into_iter()
            .map(|yaml| {
                let value: Value = yaml.into();
                if ColumnType::from(&value) != column_type {
                    panic!("the type of the values in 'invalid-values' must be the same as 'columm-type'")
                }
                value
            })
            .collect()
    });

    let output = input
        .remove(&Yaml::from_str("output"))
        .map(|yaml| parse_output(yaml.into_string().expect("'output' must be a string")))
        .unwrap_or(Output::Identifier(Ident::new("value", Span::call_site())));

    let aggregate = input
        .remove(&Yaml::from_str("aggregate"))
        .map(|yaml| {
            yaml.as_str()
                .expect("value of 'aggregate' must be a string")
                .into()
        })
        .unwrap_or(Aggregate::First);

    if aggregate == Aggregate::Average && !output_type.is_numeric() {
        panic!("'{output_type}' cannot be averaged");
    }

    ensure_empty(&input, "column");

    Column {
        title,
        column_type,
        output_type,
        null_surrogates,
        valid_values,
        on_invalid,
        on_null,
        max,
        min,
        invalid_values,
        output,
        ignore,
        aggregate,
        process_columns: vec![],
    }
}

fn parse_process(input: Yaml) -> Process {
    let mut input = input.into_hash().expect("'processes' entires must be maps");

    let name = input
        .remove(&Yaml::from_str("name"))
        .expect("process name required")
        .into_string()
        .expect("process name must be a string");

    let columns = input
        .remove(&Yaml::from_str("columns"))
        .expect("'columns' key required")
        .into_vec()
        .expect("'columns' must be an array")
        .into_iter()
        .map(parse_column)
        .collect();

    let aggregate_column = input
        .remove(&Yaml::from_str("aggregate-column"))
        .map(|yaml| {
            Ident::new(
                yaml.as_str()
                    .expect("values of 'aggregate' must be a string"),
                Span::call_site(),
            )
        });

    ensure_empty(&input, "process");

    let mut process = Process {
        name,
        columns,
        aggregate_column,
    };

    let names = process.column_names();
    let column_types = process.column_types();
    for (i, (name, column_type)) in zip(&names, column_types).enumerate() {
        for (j, column) in process.columns.iter_mut().enumerate() {
            if i == j {
                continue;
            }

            column.process_columns.push((name.clone(), column_type));
        }
    }

    if let Some(aggregate) = &process.aggregate_column {
        if !names.contains(aggregate) {
            panic!("aggregate value '{aggregate}' is not a column");
        }
    }

    let mut var_types: HashMap<Ident, ColumnType> = process
        .columns
        .iter()
        .map(|column| {
            (
                Ident::new(
                    &format!("value_{}", column.title.to_owned()),
                    Span::call_site(),
                ),
                column.output_type,
            )
        })
        .collect();
    let value_ident = Ident::new("value", Span::call_site());

    for column in &process.columns {
        var_types.insert(value_ident.clone(), column.column_type);
        let return_type = match column.output.return_type(&var_types) {
            Ok(return_type) => return_type,
            Err(message) => panic!(
                "process '{}', column '{}': {}",
                process.name, column.title, message
            ),
        };
        if return_type != column.output_type {
            panic!(
                "process '{}', column '{}': expected {}, found {}",
                process.name, column.title, column.output_type, return_type
            );
        }
        var_types.remove(&value_ident);
    }

    process
}

fn parse_program(input: Yaml, csv: Expr, string_input: bool) -> Program {
    let mut program = input.into_hash().expect("config must be a map");

    let processes = program
        .remove(&Yaml::from_str("processes"))
        .expect("'processes' key is required")
        .into_vec()
        .expect("'processes' must be an array")
        .into_iter()
        .map(parse_process)
        .collect();

    let on_title = program
        .remove(&Yaml::from_str("on-title"))
        .map(|yaml| yaml.into())
        .unwrap_or(OnTitle::Once);

    ensure_empty(&program, "program");

    Program {
        processes,
        on_title,
        csv,
        string_input,
    }
}

struct MacroInput {
    config: LitStr,
    _comma: Token![,],
    csv: Expr,
    _final_comma: Option<Token![,]>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let config: Expr = input.parse()?;
        let tokens: proc_macro::TokenStream = config.to_token_stream().into();
        let tokens = tokens.expand_expr().unwrap_or_else(|err| panic!("{err}"));
        let config = parse(tokens)?;

        let _comma = input.parse()?;
        let csv = input.parse()?;
        let _final_comma = input.parse()?;

        Ok(MacroInput {
            config,
            _comma,
            csv,
            _final_comma,
        })
    }
}

/// Cleans up and validates data.
///
/// The first argument must be either a string literal or a macro call that expands to a string literal.
/// The second argument must be an expression that resolves to a tuple of borrowed slices of options containing the values of the input file.
/// The slices must all be the same length.
/// This is more clearly explained by the examples.
///
/// # Examples
/// ```
/// # use std::iter::zip;
/// # use sanitise::sanitise;
///
/// let time = vec![Some(0), Some(15), Some(127)];
/// let pulse = vec![Some(67), Some(45), Some(132)];
/// let movement = vec![Some(0), Some(1), Some(1)];
/// let ((time_millis, pulse, movement),) = sanitise!(
///     r#"
///         processes:
///           - name: validate
///             columns:
///               - title: time
///                 column-type: integer
///               - title: pulse
///                 column-type: integer
///                 max: 100
///                 min: 40
///                 on-invalid: average
///                 valid-streak: 3
///               - title: movement
///                 column-type: integer
///                 valid-values: [0, 1]
///                 output-type: boolean
///                 output: "value == 1"
///     "#,
///     (&time, &pulse, &movement),
/// ).unwrap();
///
/// println!("time_millis,pulse,movement");
/// for ((time_millis, pulse), movement) in zip(zip(time_millis, pulse), movement) {
///     println!("{time_millis},{pulse},{movement}")
/// }
/// ```
#[proc_macro]
pub fn sanitise(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    let source = input.config.value();
    let yaml = VecDeque::from(YamlLoader::load_from_str(&source).expect("failed to parse yaml"))
        .pop_front()
        .expect("expect at least one document");

    let program = parse_program(yaml, input.csv, false);

    program.to_token_stream().into()
}

/// Cleans up and validates data from a string.
///
/// The first argument must be either a string literal or a macro call that expands to a string literal.
/// The second argument must be an expression that resolves to a string in CSV format.
///
/// # Examples
/// ```
/// # use std::{fs, iter::zip};
/// # use sanitise::sanitise_string;
///
/// let csv = "time,pulse,movement\n0,67,0\n15,45,1\n126,132,1\n";
/// let ((time_millis, pulse, movement),) = sanitise_string!(
///     r#"
///         processes:
///           - name: validate
///             columns:
///               - title: time
///                 column-type: integer
///               - title: pulse
///                 column-type: integer
///                 max: 100
///                 min: 40
///                 on-invalid: average
///                 valid-streak: 3
///               - title: movement
///                 column-type: integer
///                 valid-values: [0, 1]
///                 output-type: boolean
///                 output: "value == 1"
///     "#,
///     csv,
/// ).unwrap();
///
/// println!("time_millis,pulse,movement");
/// for ((time_millis, pulse), movement) in zip(zip(time_millis, pulse), movement) {
///     println!("{time_millis},{pulse},{movement}")
/// }
/// ```
#[proc_macro]
pub fn sanitise_string(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    let source = input.config.value();
    let yaml = VecDeque::from(YamlLoader::load_from_str(&source).expect("failed to parse yaml"))
        .pop_front()
        .expect("expect at least one document");

    let program = parse_program(yaml, input.csv, true);

    program.to_token_stream().into()
}
