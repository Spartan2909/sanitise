use crate::{
    Aggregate, BinOp, Column, ColumnType, Function, OnInvalid, OnTitle, Output, Process, Program,
    UnOp, Value,
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Index;

struct ValueList<'a>(&'a Vec<Value>);

impl<'a> ToTokens for ValueList<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut inner = TokenStream::new();
        for value in self.0 {
            inner.extend(quote!(#value.to_owned(),));
        }
        tokens.extend(quote!([ #inner ]));
    }
}

impl ToTokens for ColumnType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = match self {
            ColumnType::Bool => quote!(bool),
            ColumnType::Float => quote!(f64),
            ColumnType::Integer => quote!(i64),
            ColumnType::String => quote!(String),
        };

        tokens.extend(inner);
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::Boolean(b) => b.to_tokens(tokens),
            Value::Integer(i) => i.to_tokens(tokens),
            Value::Real(r) => r.to_tokens(tokens),
            Value::String(s) => s.to_tokens(tokens),
        }
    }
}

impl ToTokens for Function {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = match self {
            Function::Boolean(arg) => quote! { SanitiseConversions::to_bool(&((#arg)?)) },
            Function::Ceiling(arg) => quote! { Ok(sanitise_ceiling(&((#arg)?))) },
            Function::Concat(arg1, arg2) => quote! {
                Ok(sanitise_concat(&((#arg1)?), &((#arg2)?)))
            },
            Function::Floor(arg) => quote! { Ok(sanitise_floor(&((#arg)?))) },
            Function::Integer(arg) => quote! { SanitiseConversions::to_int(&((#arg)?)) },
            Function::Real(arg) => quote! { SanitiseConversions::to_float(&((#arg)?)) },
            Function::Round(arg) => quote! { Ok(sanitise_round(&((#arg)?))) },
            Function::String(arg) => quote! { SanitiseConversions::to_string(&((#arg)?)) },
        };

        tokens.extend(inner);
    }
}

impl ToTokens for BinOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = match self {
            BinOp::Add => quote!(+),
            BinOp::Sub => quote!(-),
            BinOp::Mul => quote!(*),
            BinOp::Div => quote!(/),
            BinOp::Mod => quote!(%),
            BinOp::Eq => quote!(==),
            BinOp::Ne => quote!(!=),
            BinOp::Gt => quote!(>),
            BinOp::Ge => quote!(>=),
            BinOp::Lt => quote!(<),
            BinOp::Le => quote!(<=),
        };

        tokens.extend(inner);
    }
}

impl ToTokens for UnOp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = match self {
            UnOp::Negate => quote!(-),
            UnOp::Not => quote!(!),
        };

        tokens.extend(inner);
    }
}

impl ToTokens for Output {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Output::Binary {
                left,
                operator,
                right,
            } => tokens.extend(quote! { Ok(((#left)?) #operator ((#right)?)) }),
            Output::Function(function) => function.to_tokens(tokens),
            Output::Identifier(ident) => {
                tokens.extend(quote!(Ok(#ident.unwrap().to_owned())));
            }
            Output::Literal(value) => tokens.extend(quote!(Ok(#value))),
            Output::Unary { operator, right } => tokens.extend(quote! { Ok(#operator((#right)?)) }),
        }
    }
}

impl ToTokens for Column {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let title = format!("Column_{}", self.title);
        let name = Ident::new(&title, Span::call_site());
        let state_name = Ident::new(&(title + "_State"), Span::call_site());

        let column_type = self.input_type;

        let output_type = self.output_type;

        let (state, new_state) = if self.needs_state() {
            tokens.extend(quote! {
                enum #state_name {
                    Valid,
                    Invalid {
                        missing: usize,
                        valid_streak: Vec<#output_type>,
                        last_action: Action,
                    },
                }
            });

            (
                quote! { state: #state_name, },
                quote!(state: #state_name::Valid),
            )
        } else {
            (TokenStream::new(), TokenStream::new())
        };

        let invalid_function = match &self.on_invalid {
            OnInvalid::Abort => {
                let message = format!("invalid value for column '{}': {{}}", self.title);
                quote!(Err(Interrupt::Error(format!(#message, value))))
            }
            OnInvalid::Average(_) => quote! {
                if let #state_name::Invalid { missing, valid_streak, last_action } = &mut self.state {
                    *missing += 1;
                    if !valid_streak.is_empty() {
                        *missing += valid_streak.len();
                        *valid_streak = vec![];
                    }
                    *last_action = Action::IncrementInvalid;
                } else {
                    self.state = #state_name::Invalid {
                        missing: 1,
                        valid_streak: vec![],
                        last_action: Action::IncrementInvalid
                    };
                }

                Ok(())
            },
            OnInvalid::Delete => quote!(Err(Interrupt::Delete)),
            OnInvalid::Previous(sentinel) => quote! {
                self.output.push(self.output.last().unwrap_or(&#sentinel).to_owned());
                Ok(())
            },
            OnInvalid::Sentinel(sentinel) => quote! {
                self.output.push(#sentinel.to_owned());
                Ok(())
            },
        };

        let null_function = match &self.on_null {
            OnInvalid::Abort => {
                let message = format!("unexpected null in column '{}'", self.title);
                quote!(Err(Interrupt::Error(#message.to_owned())))
            }
            OnInvalid::Average(_) => quote! {
                self.invalid(&#column_type::default())
            },
            OnInvalid::Delete => quote!(Err(Interrupt::Delete)),
            OnInvalid::Previous(sentinel) => quote! {
                self.output.push(self.output.last().unwrap_or(&#sentinel).to_owned());
                Ok(())
            },
            OnInvalid::Sentinel(sentinel) => quote! {
                self.output.push(#sentinel.to_owned());
                Ok(())
            },
        };

        let mut push_function = TokenStream::new();

        if let Some(max) = &self.max {
            push_function.extend(quote! {
                if value > &#max {
                    return self.invalid(value);
                }
            });
        }

        if let Some(min) = &self.min {
            push_function.extend(quote! {
                if value < &#min {
                    return self.invalid(value);
                }
            });
        }

        if let Some(invalid_values) = &self.invalid_values {
            let invalid_values = ValueList(invalid_values);
            push_function.extend(quote! {
                if #invalid_values.contains(value) {
                    return self.invalid(value);
                }
            });
        }

        if let Some(valid_values) = &self.valid_values {
            let valid_values = ValueList(valid_values);
            push_function.extend(quote! {
                if !#valid_values.contains(value) {
                    return self.invalid(value);
                }
            });
        }

        let output = &self.output;

        push_function.extend(quote! {
            let value = Some(value);
            self.push_valid((#output)?);
            Ok(())
        });

        let two = if self.output_type == ColumnType::Integer {
            quote!(2)
        } else {
            quote!(2.0)
        };

        let valid_function = if let OnInvalid::Average(valid_streak) = self.on_invalid {
            quote! {
                if let #state_name::Invalid { missing, valid_streak, last_action } = &mut self.state {
                    valid_streak.push(value);
                    *last_action = Action::AppendValid;
                    if valid_streak.len() >= #valid_streak {
                        let before_invalid_streak = self.output.last().unwrap_or(&valid_streak[0]);
                        let after_invalid_streak = &valid_streak[0];
                        let average = (before_invalid_streak + after_invalid_streak) / &#two;
                        for _ in 0..(*missing) {
                            self.output.push(average.to_owned())
                        }
                        self.output.extend(valid_streak.iter().cloned());
                        self.state = #state_name::Valid;
                    }
                } else {
                    self.output.push(value);
                }
            }
        } else {
            quote! {
                self.output.push(value);
            }
        };

        let undo_function = if let OnInvalid::Average(_) = self.on_invalid {
            quote! {
                if let #state_name::Invalid { missing, valid_streak, last_action } = &mut self.state {
                    if *last_action == Action::AppendValid {
                        valid_streak.pop();
                    } else {
                        *missing -= 1;
                        if *missing == 0 {
                            self.state = #state_name::Valid;
                        }
                    }
                } else {
                    self.output.pop();
                }
            }
        } else {
            quote!(self.output.pop();)
        };

        let aggregate_function = match self.aggregate {
            Aggregate::Average => quote! {
                let mut sum = self.output[end_index].clone();
                for item in &self.output[start_index..end_index] {
                    sum += item.clone();
                }
                sum / (end_index - start_index + 1) as #output_type
            },
            Aggregate::First => quote!(self.output[start_index].clone()),
            Aggregate::Last => quote!(self.output[end_index].clone()),
        };

        let finish_function = if let OnInvalid::Average(_) = self.on_invalid {
            quote! {
                if let #state_name::Invalid { missing, valid_streak, last_action } = &mut self.state {
                    *missing += valid_streak.len();
                    let last_valid = if let Some(last) = self.output.last() {
                        last.to_owned()
                    } else {
                        return Err(Interrupt::Error("No valid values".to_string()));
                    };
                    for _ in 0..(*missing) {
                        self.output.push(last_valid.clone());
                    }
                }

                Ok(())
            }
        } else {
            quote!(Ok(()))
        };

        let mut push_function_params = TokenStream::new();
        for (name, column_type) in &self.process_columns {
            push_function_params.extend(quote!(#name: Option<&#column_type>,));
        }

        tokens.extend(quote! {
            struct #name {
                output: Vec<#output_type>,
                #state
            }

            impl #name {
                #[inline(always)]
                fn new() -> #name {
                    #name { output: vec![], #new_state }
                }

                fn invalid(&mut self, value: &#column_type) -> Result<(), Interrupt> {
                    #invalid_function
                }

                fn null(&mut self) -> Result<(), Interrupt> {
                    #null_function
                }

                #[inline(always)]
                fn push_valid(&mut self, value: #output_type) {
                    #valid_function
                }

                #[inline(always)]
                fn push(&mut self, value: &#column_type, #push_function_params) -> Result<(), Interrupt> {
                    #push_function
                }

                fn undo(&mut self) {
                    #undo_function
                }

                fn aggregate(&self, start_index: usize, end_index: usize) -> #output_type {
                    #aggregate_function
                }

                #[inline(always)]
                fn finish(&mut self) -> Result<(), Interrupt> {
                    #finish_function
                }
            }
        });
    }
}

impl ToTokens for Process {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut inner = TokenStream::new();

        for column in &self.columns {
            if !column.ignore {
                column.to_tokens(&mut inner);
            }
        }

        let mut automata_initialisation = quote! {
            if file.0.is_empty() {
                return Err(("Empty file".to_string(), 1));
            }
        };
        let mut automata_details = vec![];
        for (i, (struct_name, ignore)) in self
            .column_names()
            .into_iter()
            .zip(self.ignores())
            .enumerate()
        {
            if ignore {
                automata_details.push(None);
            } else {
                let automaton_name = Ident::new(&format!("automaton_{i}"), Span::call_site());
                let title = format!("Column_{struct_name}");
                let struct_name = Ident::new(&title, Span::call_site());
                automata_initialisation
                    .extend(quote!(let mut #automaton_name = #struct_name::new();));
                automata_details.push(Some((
                    automaton_name,
                    self.columns[i].null_surrogates.clone(),
                )));
            }
        }

        let mut automata_feed = TokenStream::new();
        let mut undo = TokenStream::new();
        let mut finish_automata = TokenStream::new();
        let mut get_returns = TokenStream::new();
        let mut return_value = TokenStream::new();
        let mut result_indexes = vec![];
        for (i, details) in automata_details.iter().enumerate() {
            let mut args = TokenStream::new();
            for j in 0..automata_details.len() {
                if i == j {
                    continue;
                }

                let j = Index::from(j);
                args.extend(quote!(file.#j[i].as_ref(),));
            }

            if let Some((automaton_name, null_surrogate)) = details {
                result_indexes.push(i);

                let index = Index::from(i);

                let push = quote! {
                    if let Err(interrupt) = #automaton_name.push(tmp, #args) {
                        match interrupt {
                            Interrupt::Delete => {
                                #undo
                                continue;
                            }
                            Interrupt::Error(s) => return Err((s, i + 1)),
                        }
                    }
                };

                let on_null = quote! {
                    if let Err(interrupt) = #automaton_name.null() {
                        match interrupt {
                            Interrupt::Delete => {
                                #undo
                                continue;
                            }
                            Interrupt::Error(s) => return Err((s, i + 1)),
                        }
                    }
                };

                let push = if let Some(surrogates) = null_surrogate {
                    let surrogates = ValueList(surrogates);
                    quote! {
                        if #surrogates.contains(&tmp) {
                            #on_null
                        } else {
                            #push
                        }
                    }
                } else {
                    push
                };

                automata_feed.extend(quote! {
                    if let Some(tmp) = &(file.#index)[i] {
                        #push
                    }
                    else {
                        #on_null
                    }
                });

                undo.extend(quote!(#automaton_name.undo();));

                let result_name = Ident::new(&format!("result_{i}"), Span::call_site());
                finish_automata.extend(quote! {
                    if let Err(interrupt) = #automaton_name.finish() {
                        return Err((interrupt.extract_error(), file.0.len()));
                    };
                });
                get_returns.extend(quote!(let #result_name = #automaton_name.output;));
                return_value.extend(quote!(#result_name,));
            }
        }

        if let Some(aggregate) = &self.aggregate_column {
            get_returns = TokenStream::new();

            let aggregate_automaton_index = self
                .column_names()
                .iter()
                .position(|x| x == aggregate)
                .unwrap_or_else(|| {
                    panic!("internal error: invalid aggregate target - '{aggregate}'")
                });
            let aggregate_automaton_name = Ident::new(
                &format!("automaton_{aggregate_automaton_index}"),
                Span::call_site(),
            );

            let mut new_result_names = vec![];
            for index in &result_indexes {
                let new_result_name = Ident::new(&format!("new_result_{index}"), Span::call_site());
                get_returns.extend(quote!(let mut #new_result_name = vec![];));
                let result_name = Ident::new(&format!("result_{index}"), Span::call_site());
                let automaton_name = Ident::new(&format!("automaton_{index}"), Span::call_site());
                new_result_names.push((
                    new_result_name,
                    automaton_name,
                    result_name,
                    *index == aggregate_automaton_index,
                ));
            }

            let mut get_aggregates = TokenStream::new();
            for (result_name, automaton_name, _, is_aggregate) in &new_result_names {
                get_aggregates.extend(if *is_aggregate {
                    quote!(#result_name.push(run_value.to_owned());)
                } else {
                    quote!(#result_name.push(#automaton_name.aggregate(start_index, i - 1));)
                });
            }

            get_returns.extend(quote! {
                let mut start_index = 0;
                let mut run_value = &#aggregate_automaton_name.output[0];
                for (i, current_value) in #aggregate_automaton_name.output.iter().enumerate() {
                    if current_value != run_value {
                        #get_aggregates
                        start_index = i;
                        run_value = current_value;
                    }
                }
            });

            for (new_result_name, _, result_name, _) in &new_result_names {
                get_returns.extend(quote!(let #result_name = #new_result_name;));
            }
        }

        let mut input_type = TokenStream::new();
        let mut parse_return = TokenStream::new();
        for column in &self.columns {
            let column_type = column.input_type;
            input_type.extend(quote!(&[Option<#column_type>],));
            parse_return.extend(quote!(Vec<Option<#column_type>>,));
        }

        let mut parse_function_declarations = TokenStream::new();
        let num_columns = automata_details.len();
        let mut parse_function_body = quote! {
            if line.len() != #num_columns {
                return Err((format!("Invalid line length: {}", line.len()), i))
            }
        };
        let mut parse_function_return = TokenStream::new();
        for i in 0..num_columns {
            let column_name = Ident::new(&format!("column_{i}"), Span::call_site());
            parse_function_declarations.extend(quote!(let mut #column_name = vec![];));
            parse_function_body.extend(quote! {
                if line[#i].is_empty() {
                    #column_name.push(None);
                } else {
                    #column_name.push(match line[#i].parse() {
                        Ok(v) => Some(v),
                        Err(_) => return Err((format!("failed to parse {}", line[#i]), i)),
                    });
                }
            });
            parse_function_return.extend(quote!(#column_name,));
        }

        let signature = self.signature();
        inner.extend(quote! {
            pub(super) fn process(file: (#input_type)) -> Result<#signature, (String, usize)> {
                #automata_initialisation

                for i in 0..(file.0.len()) {
                    #automata_feed
                }

                #finish_automata
                #get_returns

                Ok((#return_value))
            }
            pub(super) fn parse(file: &[Vec<&str>]) -> Result<(#parse_return), (String, usize)> {
                #parse_function_declarations

                for (i, line) in file.iter().enumerate() {
                    #parse_function_body
                }

                Ok((#parse_function_return))
            }
        });

        tokens.extend(quote! {
            use super::*;
            #inner
        });
    }
}

impl ToTokens for Program {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let csv = &self.csv;

        #[cfg(feature = "benchmark")]
        let mut inner = quote! {
            #[cfg(debug_assertions)]
            use ::std::{time::Instant, println};
        };

        #[cfg(not(feature = "benchmark"))]
        let mut inner = TokenStream::new();

        let process_function_input_type = if self.string_input {
            quote!(&[Vec<&str>])
        } else {
            let mut file_type = TokenStream::new();

            for column_type in self.processes[0].column_types() {
                file_type.extend(quote!(&[Option<#column_type>],));
            }

            quote!((#file_type))
        };

        let main_function_input_type = if self.string_input {
            quote!(&str)
        } else {
            #[allow(clippy::redundant_clone)]
            process_function_input_type.clone()
        };

        inner.extend(quote! {
            extern crate alloc;
            use ::core::prelude::rust_2021::*;
            use alloc::{boxed::Box, collections::VecDeque, vec, vec::Vec};

            enum Interrupt {
                Delete,
                Error(String),
            }

            impl Interrupt {
                fn extract_error(self) -> String {
                    if let Interrupt::Error(message) = self {
                        message
                    } else {
                        panic!("attempted to extract error from 'Delete'")
                    }
                }
            }

            #[derive(Clone, Copy, PartialEq)]
            enum Action {
                AppendValid,
                IncrementInvalid,
            }

            trait SanitiseConversions {
                fn to_bool(&self) -> Result<bool, Interrupt>;
                fn to_float(&self) -> Result<f64, Interrupt>;
                fn to_int(&self) -> Result<i64, Interrupt>;
                fn to_string(&self) -> Result<String, Interrupt>;
            }

            impl SanitiseConversions for bool {
                #[inline(always)]
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(*self)
                }

                #[inline(always)]
                fn to_float(&self) -> Result<f64, Interrupt> {
                    if *self {
                        Ok(1.0)
                    } else {
                        Ok(0.0)
                    }
                }

                #[inline(always)]
                fn to_int(&self) -> Result<i64, Interrupt> {
                    if *self {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }

                #[inline(always)]
                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }

            impl SanitiseConversions for f64 {
                #[inline(always)]
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(self != &0.0)
                }

                #[inline(always)]
                fn to_float(&self) -> Result<f64, Interrupt> {
                    Ok(*self)
                }

                #[inline(always)]
                fn to_int(&self) -> Result<i64, Interrupt> {
                    Ok(*self as i64)
                }

                #[inline(always)]
                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }

            impl SanitiseConversions for i64 {
                #[inline(always)]
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(self != &0)
                }

                #[inline(always)]
                fn to_float(&self) -> Result<f64, Interrupt> {
                    Ok(*self as f64)
                }

                #[inline(always)]
                fn to_int(&self) -> Result<i64, Interrupt> {
                    Ok(*self)
                }

                #[inline(always)]
                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }

            impl SanitiseConversions for String {
                #[inline(always)]
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(!self.is_empty())
                }

                fn to_float(&self) -> Result<f64, Interrupt> {
                    if let Ok(n) = self.parse() {
                        Ok(n)
                    } else {
                        let message = format!("invalid base for float: '{self}'");
                        Err(Interrupt::Error(message))
                    }
                }

                fn to_int(&self) -> Result<i64, Interrupt> {
                    if let Ok(n) = self.parse() {
                        Ok(n)
                    } else {
                        let message = format!("invalid base for int: '{self}'");
                        Err(Interrupt::Error(message))
                    }
                }

                #[inline(always)]
                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(self.to_owned())
                }
            }

            #[inline(always)]
            fn sanitise_ceiling(value: &f64) -> f64 {
                (*value).ceil()
            }

            #[inline(always)]
            fn sanitise_floor(value: &f64) -> f64 {
                (*value).floor()
            }

            #[inline(always)]
            fn sanitise_round(value: &f64) -> f64 {
                (*value).round()
            }

            #[inline(always)]
            fn sanitise_concat(value1: &str, value2: &str) -> String {
                let mut output = String::with_capacity(value1.len() + value2.len());
                output.push_str(value1);
                output.push_str(value2);
                output
            }
        });

        let mut process_data = vec![];
        let mut signature = TokenStream::new();
        for (i, process) in self.processes.iter().enumerate() {
            let name = Ident::new(&process.name, Span::mixed_site());

            inner.extend(quote!(mod #name { #process }));

            if i == 0 {
                inner.extend(quote! {
                    use #name::parse as parse_file;
                });
            }

            process_data.push((name, process.column_names(), process.ignores()));

            signature.extend(process.signature());
            signature.extend(quote!(,));
        }
        let signature = quote!((#signature));
        let main_signature = match self.on_title {
            #[expect(
                clippy::redundant_clone,
                reason = "`signature` is used when constructing `process`"
            )]
            OnTitle::Combine | OnTitle::Once => signature.clone(),
            OnTitle::Split => quote!(Vec<#signature>),
        };
        let header = self.processes[0].header(false);

        let mut initial_assignment_target = TokenStream::new();
        let mut args = TokenStream::new();
        for i in 0..process_data[0].1.len() {
            let input_name = Ident::new(&format!("item_{i}"), Span::call_site());
            initial_assignment_target.extend(quote!(#input_name,));
            args.extend(quote!(&#input_name,));
        }

        #[cfg(feature = "benchmark")]
        let mut process_function = quote! {
            let start_time = Instant::now();
        };

        #[cfg(not(feature = "benchmark"))]
        let mut process_function = TokenStream::new();

        if self.string_input {
            process_function.extend(quote! {
                let (#initial_assignment_target) = parse_file(file)?;
            });
        } else {
            process_function.extend(quote! {
                let (#initial_assignment_target) = file;
            });
        }

        let mut process_function_return = TokenStream::new();

        let mut args = quote!((#args));
        for (i, (process_name, column_names, ignores)) in process_data.iter().enumerate() {
            let mut assignment_target = TokenStream::new();
            let mut inputs = TokenStream::new();
            let mut returns = TokenStream::new();
            for (column_name, &ignored) in column_names.iter().zip(ignores) {
                if !ignored {
                    let column_name = Ident::new(&format!("{column_name}_{i}"), Span::call_site());
                    assignment_target.extend(quote!(#column_name,));
                    inputs.extend(
                        quote!(&Vec::from_iter(#column_name.iter().map(|x| Some(x.to_owned()))),),
                    );
                    returns.extend(quote!(#column_name,));
                }
            }
            process_function
                .extend(quote! { let (#assignment_target) = #process_name::process(#args)?; });

            args = quote!((#inputs));

            process_function_return.extend(quote!((#returns),));
        }

        #[cfg(feature = "benchmark")]
        process_function.extend(quote! {
            println!("Process function finished: {}ms", start_time.elapsed().as_millis());
        });

        #[cfg(feature = "benchmark")]
        let start_of_main = quote! {
            let start_time = Instant::now();
        };

        #[cfg(not(feature = "benchmark"))]
        let start_of_main = TokenStream::new();

        let process_files = match self.on_title {
            OnTitle::Combine => quote! {
                let mut files: VecDeque<_> = files.into();
                files.pop_front(); // Discard the first empty vec
                let mut file = files.pop_front().unwrap(); // Guaranteed to exist by check in main function
                for file_section in files.iter_mut() {
                    file.append(file_section);
                }
                let result = process(&file);
            },
            OnTitle::Once => quote! {
                if files.len() > 2 {
                    return Err(("Found extra set of headers".to_owned(), files[1].len() + 1))
                }

                let result = process(&files[1]);
            },
            OnTitle::Split => quote! {
                let result = files[1..].iter().map(|file| process(file)).collect();
            },
        };

        #[cfg(feature = "benchmark")]
        let file_gen_benchmark = quote! {
            println!("Split input into files: {}ms", start_time.elapsed().as_millis());
        };

        #[cfg(not(feature = "benchmark"))]
        let file_gen_benchmark = TokenStream::new();

        let main_body = if self.string_input {
            quote! {
                let files = get_files(csv);

                if files.len() < 2 {
                    return Err(("found no headers".to_string(), 1));
                }

                #file_gen_benchmark

                #process_files
            }
        } else {
            quote! {
                let result = process(csv);
            }
        };

        #[cfg(feature = "benchmark")]
        let main_return = quote! {
            println!("Main function finished: {}ms", start_time.elapsed().as_millis());
            result
        };

        #[cfg(not(feature = "benchmark"))]
        let main_return = quote!(result);

        inner.extend(quote! {
            fn get_files(csv: &str) -> Vec<Vec<Vec<&str>>> {
                let mut lines: Vec<&str> = csv
                    .split('\n')
                    .map(|s| {
                        s.strip_suffix("\r")
                            .unwrap_or(s)
                    })
                    .collect();
                if lines.last().is_some_and(|line| line.is_empty()) {
                    lines.pop();
                }

                lines
                    .split(|&line| line == #header)
                    .map(|file| {
                        file
                            .iter()
                            .map(|line| {
                                line.split(',').collect()
                            })
                            .collect()
                    })
                    .collect()
            }

            #[inline(always)]
            fn process(file: #process_function_input_type) -> Result<#signature, (String, usize)> {
                #process_function

                Ok((#process_function_return))
            }

            #[inline(always)]
            pub(super) fn main(csv: #main_function_input_type) -> Result<#main_signature, (String, usize)> {
                #start_of_main

                #main_body

                #main_return
            }
        });

        tokens.extend(quote! { {
            mod __sanitise { #inner }
            __sanitise::main(#csv)
        } });
    }
}
