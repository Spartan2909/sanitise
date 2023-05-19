#![feature(proc_macro_expand)]
#![doc = include_str!("../README.md")]

mod output;
use output::parse_output;
mod to_tokens;

use std::{collections::VecDeque, fmt, iter::zip};

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, LitStr, Result, Token,
};
use yaml_rust::{yaml::Hash, Yaml, YamlLoader};

#[derive(Debug, Clone, Copy, PartialEq)]
enum ColumnType {
    Bool,
    Float,
    Integer,
    String,
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

impl From<String> for ColumnType {
    fn from(value: String) -> Self {
        ColumnType::from(value.as_str())
    }
}

impl From<&ColumnType> for Ident {
    fn from(value: &ColumnType) -> Self {
        let string = match value {
            ColumnType::Bool => "bool",
            ColumnType::Float => "f64",
            ColumnType::Integer => "i64",
            ColumnType::String => "String",
        };

        Ident::new(string, Span::mixed_site())
    }
}

impl From<ColumnType> for Ident {
    fn from(value: ColumnType) -> Self {
        (&value).into()
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
enum UnOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
enum Function {
    Boolean(Box<Output>),
    Ceiling(Box<Output>),
    Floor(Box<Output>),
    Integer(Box<Output>),
    Real(Box<Output>),
    Round(Box<Output>),
    String(Box<Output>),
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

#[derive(Debug, Clone)]
struct Column {
    title: String,
    column_type: ColumnType,
    output_type: ColumnType,
    null_surrogate: Option<Value>,
    valid_values: Option<Vec<Value>>,
    on_invalid: OnInvalid,
    on_null: OnInvalid,
    max: Option<Value>,
    min: Option<Value>,
    output: Output,
    ignore: bool,
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
}

impl Process {
    fn signiature(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        for column in &self.columns {
            if column.ignore {
                continue;
            }

            let output_type: Ident = column.output_type.into();

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
            "once" => OnTitle::Once,
            "split" => OnTitle::Split,
            _ => panic!("invalid value for title: '{on_title}'"),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct ProgramDebug<'a> {
    processes: &'a Vec<Process>,
    on_title: &'a OnTitle,
}

#[derive(Clone)]
struct Program {
    processes: Vec<Process>,
    on_title: OnTitle,
    csv: Expr,
}

impl fmt::Debug for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        ProgramDebug {
            processes: &self.processes,
            on_title: &self.on_title,
        }
        .fmt(f)
    }
}

fn ensure_empty(hash: &Hash, map_name: &str) {
    if !hash.is_empty() {
        panic!(
            "unexpected key '{}' in {}",
            hash.keys().next().unwrap().as_str().unwrap(),
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

    let output_type = input
        .remove(&Yaml::from_str("output-type"))
        .map(|yaml| {
            yaml.into_string()
                .expect("column type must be a string")
                .into()
        })
        .unwrap_or(column_type);

    let null_surrogate = input
        .remove(&Yaml::from_str("null-surrogate"))
        .map(|yaml| yaml.into());

    let valid_values = input.remove(&Yaml::from_str("valid-values")).map(|yaml| {
        yaml.into_vec()
            .expect("'valid-values' must be an array")
            .into_iter()
            .map(|yaml| yaml.into())
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

    let max = input.remove(&Yaml::from_str("max")).map(|yaml| yaml.into());

    let min = input.remove(&Yaml::from_str("min")).map(|yaml| yaml.into());

    let output = input
        .remove(&Yaml::from_str("output"))
        .map(|yaml| parse_output(yaml.into_string().expect("'output' must be a string")))
        .unwrap_or(Output::Identifier(Ident::new("value", Span::call_site())));

    let ignore = input
        .remove(&Yaml::from_str("ignore"))
        .map(|yaml| yaml.as_bool().expect("'ignore' must be a Boolean"))
        .unwrap_or(false);

    ensure_empty(&input, "column");

    Column {
        title,
        column_type,
        output_type,
        null_surrogate,
        valid_values,
        on_invalid,
        on_null,
        max,
        min,
        output,
        ignore,
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

    ensure_empty(&input, "process");

    let mut process = Process { name, columns };

    let names = process.column_names();
    let column_types = process.column_types();
    for (i, (name, column_type)) in zip(names, column_types).enumerate() {
        for (j, column) in process.columns.iter_mut().enumerate() {
            if i == j {
                continue;
            }

            column.process_columns.push((name.clone(), column_type));
        }
    }

    process
}

fn parse_program(input: Yaml, csv: Expr) -> Program {
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
    }
}

struct MacroInput {
    config: LitStr,
    _comma: Token![,],
    csv: Expr,
    _final_comma: Option<Token![,]>,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
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
/// The second argument must be an expression that resolves to a string in CSV format.
///
/// # Examples
/// ```
/// # use std::{fs, iter::zip};
/// # use sanitise::sanitise;
///
/// let csv = "time,pulse,movement\n0,67,0\n15,45,1\n126,132,1\n";
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
///     csv,
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
        .unwrap();

    let program = parse_program(yaml, input.csv);

    program.to_token_stream().into()
}
