#![feature(proc_macro_expand)]

mod output;
use output::parse_output;

use std::collections::VecDeque;

extern crate proc_macro;

use quote::{quote, ToTokens};
use syn::{
    parse,
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, LitStr, Result, Token,
};
use yaml_rust::{yaml::Hash, Yaml, YamlLoader};

#[derive(Debug, Clone, Copy)]
enum ColumnType {
    Float,
    Integer,
    String,
}

impl From<&str> for ColumnType {
    fn from(value: &str) -> Self {
        match value {
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
enum OnInvalid {
    Abort,
    Average,
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
        "average" => OnInvalid::Average,
        "delete" => OnInvalid::Delete,
        "previous" => {
            let key = format!("{kind}-sentinel");
            let sentinel = hash
                .remove(&Yaml::from_str(&key))
                .unwrap_or_else(|| {
                    panic!("'previous' option for on-{kind} requires key '{kind}-sentinel")
                })
                .into();
            OnInvalid::Previous(sentinel)
        }
        "sentinel" => {
            let key = format!("{kind}-sentinel");
            let sentinel = hash
                .remove(&Yaml::from_str(&key))
                .unwrap_or_else(|| {
                    panic!("'sentinel' option for on-{kind} requires key '{kind}-sentinel")
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
    Identifier(String),
    Literal(Value),
    Negate(Box<Output>),
}

#[derive(Debug, Clone)]
struct Column {
    title: String,
    kind: ColumnType,
    output_kind: ColumnType,
    null_surrogate: Option<Value>,
    valid_values: Option<Vec<Value>>,
    on_invalid: OnInvalid,
    on_null: OnInvalid,
    max: Option<Value>,
    min: Option<Value>,
    output: Output,
    ignore: bool,
}

#[derive(Debug, Clone)]
struct Process {
    name: String,
    columns: Vec<Column>,
}

#[derive(Debug, Clone)]
struct Program(Vec<Process>);

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

    let kind = input
        .remove(&Yaml::from_str("type"))
        .expect("column kind required")
        .into_string()
        .expect("column type must be a string")
        .into();

    let output_kind = input
        .remove(&Yaml::from_str("output-type"))
        .map(|yaml| {
            yaml.into_string()
                .expect("column type must be a string")
                .into()
        })
        .unwrap_or(kind);

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

    let max = input.remove(&Yaml::from_str("max")).map(|yaml| yaml.into());

    let min = input.remove(&Yaml::from_str("min")).map(|yaml| yaml.into());

    let output = input
        .remove(&Yaml::from_str("output"))
        .map(|yaml| parse_output(yaml.into_string().expect("'output' must be a string")))
        .unwrap_or(Output::Identifier("value".to_owned()));

    let ignore = input
        .remove(&Yaml::from_str("ignore"))
        .map(|yaml| yaml.as_bool().expect("'ignore' must be a Boolean"))
        .unwrap_or(false);

    ensure_empty(&input, "column");

    Column {
        title,
        kind,
        output_kind,
        null_surrogate,
        valid_values,
        on_invalid,
        on_null,
        max,
        min,
        output,
        ignore,
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
        .map(|yaml| parse_column(yaml))
        .collect();

    ensure_empty(&input, "process");

    Process { name, columns }
}

fn parse_program(input: Yaml) -> Program {
    let mut program = input.into_hash().expect("config must be a map");

    let processes = program
        .remove(&Yaml::from_str("processes"))
        .expect("'processes' key is required")
        .into_vec()
        .expect("'processes' must be an array")
        .into_iter()
        .map(|yaml| parse_process(yaml))
        .collect();

    ensure_empty(&program, "program");

    Program(processes)
}

struct MacroInput {
    config: LitStr,
    _comma: Token![,],
    csv: Expr,
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let config: Expr = input.parse()?;
        let _comma = input.parse()?;
        let csv = input.parse()?;

        let tokens: proc_macro::TokenStream = config.to_token_stream().into();
        let tokens = tokens.expand_expr().unwrap_or_else(|err| panic!("{err}"));

        let config = parse(tokens)?;

        Ok(MacroInput {
            config,
            _comma,
            csv,
        })
    }
}

#[proc_macro]
pub fn sanitise(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    let source = input.config.value();
    let yaml = VecDeque::from(YamlLoader::load_from_str(&source).expect("failed to parse yaml"))
        .pop_front()
        .unwrap();

    let program = parse_program(yaml);

    let output = format!("{:#?}", program);

    quote!(#output).into()
}
