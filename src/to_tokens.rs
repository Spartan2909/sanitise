use crate::{
    BinOp, Column, ColumnType, Function, OnInvalid, OnTitle, Output, Process, Program, UnOp, Value,
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Index;

struct ValueList<'a>(&'a Vec<Value>);

impl<'a> ToTokens for ValueList<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let mut inner = TokenStream::new();
        for (i, value) in self.0.iter().enumerate() {
            value.to_tokens(&mut inner);
            if i + 1 < self.0.len() {
                inner.extend(quote!(,));
            }
        }
        tokens.extend(quote!([ #inner ]));
    }
}

impl ToTokens for ColumnType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident: Ident = self.into();
        ident.to_tokens(tokens);
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

        let column_type: Ident = self.column_type.into();

        let output_type: Ident = self.output_type.into();

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
            })
        }

        if let Some(min) = &self.min {
            push_function.extend(quote! {
                if value < &#min {
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

                Ok(self.output)
            }
        } else {
            quote!(Ok(self.output))
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
                fn new() -> #name {
                    #name { output: vec![], #new_state }
                }

                fn invalid(&mut self, value: &#column_type) -> Result<(), Interrupt> {
                    #invalid_function
                }

                fn null(&mut self) -> Result<(), Interrupt> {
                    #null_function
                }

                fn push_valid(&mut self, value: #output_type) {
                    #valid_function
                }

                fn push(&mut self, value: &#column_type, #push_function_params) -> Result<(), Interrupt> {
                    #push_function
                }

                fn undo(&mut self) {
                    #undo_function
                }

                fn finish(mut self) -> Result<Vec<#output_type>, Interrupt> {
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

        let mut function_body = quote! {
            if file.0.is_empty() {
                return Err(("Empty file".to_string(), 1));
            }
        };
        let mut automata_names = vec![];
        for (i, (struct_name, ignore)) in self
            .column_names()
            .into_iter()
            .zip(self.ignores())
            .enumerate()
        {
            if ignore {
                automata_names.push(None);
            } else {
                let automaton_name = Ident::new(&format!("automaton_{i}"), Span::call_site());
                let title = format!("Column_{struct_name}");
                let struct_name = Ident::new(&title, Span::call_site());
                function_body.extend(quote!(let mut #automaton_name = #struct_name::new();));
                automata_names.push(Some((
                    automaton_name,
                    self.columns[i].null_surrogate.clone(),
                )));
            }
        }

        let mut automata_feed = TokenStream::new();
        let mut undo = TokenStream::new();
        for (i, details) in automata_names.iter().enumerate() {
            let mut args = TokenStream::new();
            for j in 0..automata_names.len() {
                if i == j {
                    continue;
                }

                let j = Index::from(j);
                args.extend(quote!(file.#j[i].as_ref(),));
            }

            let i = Index::from(i);
            if let Some((automaton_name, null_surrogate)) = details {
                let push = quote! {
                    if let Err(interrupt) = #automaton_name.push(&tmp, #args) {
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

                let push = if let Some(surrogate) = null_surrogate {
                    quote! {
                        if tmp == #surrogate.to_owned() {
                            #on_null
                        } else {
                            #push
                        }
                    }
                } else {
                    push
                };

                automata_feed.extend(quote! {
                    if let Some(tmp) = (file.#i)[i] {
                        #push
                    }
                    else {
                        #on_null
                    }
                });

                undo.extend(quote!(#automaton_name.undo();));
            }
        }

        function_body.extend(quote! {
            for i in 0..(file.0.len()) {
                #automata_feed
            }
        });

        let mut input_type = TokenStream::new();
        let mut parse_return = TokenStream::new();
        for column in &self.columns {
            let column_type: Ident = column.column_type.into();
            input_type.extend(quote!(&[Option<#column_type>],));
            parse_return.extend(quote!(Vec<Option<#column_type>>,));
        }

        let mut return_value = TokenStream::new();
        for (i, details) in automata_names.iter().enumerate() {
            if let Some((automaton_name, _)) = details {
                let result_name = Ident::new(&format!("result_{i}"), Span::call_site());
                function_body.extend(quote! {
                    let #result_name = match #automaton_name.finish() {
                        Ok(v) => v,
                        Err(interrupt) => Err((interrupt.extract_error(), file.0.len()))?,
                    };
                });
                return_value.extend(quote!(#result_name,));
            }
        }

        function_body.extend(quote!(Ok((#return_value))));

        let mut parse_function_declarations = TokenStream::new();
        let num_columns = automata_names.len();
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

        let signiature = self.signiature();
        inner.extend(quote! {
            let process = |file: (#input_type)| -> Result<#signiature, (String, usize)> {
                #function_body
            };
            let parse = |file: &[Vec<String>]| -> Result<(#parse_return), (String, usize)> {
                #parse_function_declarations
                for (i, line) in file.iter().enumerate() {
                    #parse_function_body
                }
                Ok((#parse_function_return))
            };
            (process, parse)
        });

        tokens.extend(quote! { { #inner } });
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

        inner.extend(quote! {
            extern crate alloc;
            use ::core::prelude::rust_2021::*;
            use alloc::{boxed::Box, vec, vec::Vec};

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
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(*self)
                }

                fn to_float(&self) -> Result<f64, Interrupt> {
                    if *self {
                        Ok(1.0)
                    } else {
                        Ok(0.0)
                    }
                }

                fn to_int(&self) -> Result<i64, Interrupt> {
                    if *self {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }

                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }

            impl SanitiseConversions for f64 {
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(self != &0.0)
                }

                fn to_float(&self) -> Result<f64, Interrupt> {
                    Ok(*self)
                }

                fn to_int(&self) -> Result<i64, Interrupt> {
                    Ok(*self as i64)
                }

                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }

            impl SanitiseConversions for i64 {
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(self != &0)
                }

                fn to_float(&self) -> Result<f64, Interrupt> {
                    Ok(*self as f64)
                }

                fn to_int(&self) -> Result<i64, Interrupt> {
                    Ok(*self)
                }

                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }

            impl SanitiseConversions for String {
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

                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(self.to_owned())
                }
            }

            fn sanitise_ceiling(value: &f64) -> f64 {
                (*value).ceil()
            }

            fn sanitise_floor(value: &f64) -> f64 {
                (*value).floor()
            }

            fn sanitise_round(value: &f64) -> f64 {
                (*value).round()
            }
        });

        let mut process_data = vec![];
        let mut signiature = TokenStream::new();
        for (i, process) in self.processes.iter().enumerate() {
            let name = format!("process_{}", &process.name);
            let name = Ident::new(&name, Span::mixed_site());

            if i == 0 {
                inner.extend(quote! { let (#name, parse_file) = #process; });
            } else {
                inner.extend(quote! { let (#name, _) = #process; });
            }

            process_data.push((name, process.column_names(), process.ignores()));

            signiature.extend(process.signiature());
            signiature.extend(quote!(,));
        }
        let signiature = quote!((#signiature));
        let main_signiature = if self.on_title == OnTitle::Split {
            quote!(Vec<#signiature>)
        } else {
            // Clippy incorrectly detects this as redundant, but it is used when constructing `process`
            #[allow(clippy::redundant_clone)]
            signiature.clone()
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

        process_function.extend(quote! {
            let (#initial_assignment_target) = parse_file(file)?;
        });
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
            process_function.extend(quote! { let (#assignment_target) = #process_name(#args)?; });

            args = quote!((#inputs));

            process_function_return.extend(quote!((#returns),));
        }

        #[cfg(feature = "benchmark")]
        process_function.extend(quote! {
            println!("Process function finished: {}ms", start_time.elapsed().as_millis());
        });

        process_function.extend(quote!(Ok((#process_function_return))));

        #[cfg(feature = "benchmark")]
        let start_of_main = quote! {
            let start_time = Instant::now();
        };

        #[cfg(not(feature = "benchmark"))]
        let start_of_main = TokenStream::new();

        let mut end_of_main = if self.on_title == OnTitle::Split {
            quote! {
                let result = files[1..].iter().map(|file| process(file)).collect();
            }
        } else {
            quote! {
                if files.len() > 2 {
                    return Err(("Found extra set of headers".to_owned(), files[1].len() + 1))
                }

                let result = process(&files[1]);
            }
        };

        #[cfg(feature = "benchmark")]
        end_of_main.extend(quote! {
            println!("Main function finished: {}ms", start_time.elapsed().as_millis());
            result
        });

        #[cfg(not(feature = "benchmark"))]
        end_of_main.extend(quote!(result));

        #[cfg(feature = "benchmark")]
        let file_gen_benchmark = quote! {
            println!("Split input into files: {}ms", start_time.elapsed().as_millis());
        };

        #[cfg(not(feature = "benchmark"))]
        let file_gen_benchmark = TokenStream::new();

        inner.extend(quote! {
            let process = |file: &[Vec<String>]| -> Result<#signiature, (String, usize)> {
                #process_function
            };

            let main = |csv: String| -> Result<#main_signiature, (String, usize)> {
                #start_of_main

                let mut lines: Vec<String> = csv
                    .split('\n')
                    .map(|s| {
                        s.strip_suffix("\r")
                            .unwrap_or(s)
                            .to_owned()
                    })
                    .collect();
                if let Some(line) = lines.last() {
                    if line.is_empty() {
                        lines.pop();
                    }
                }

                let files: Vec<Vec<Vec<String>>> = lines
                    .split(|line| line == #header)
                    .map(|file| {
                        file
                            .iter()
                            .map(|line| {
                                line.split(',').map(|item| item.to_owned()).collect()
                            })
                            .collect()
                    })
                    .collect();

                #file_gen_benchmark

                #end_of_main
            };
            main(#csv)
        });

        tokens.extend(quote! { { #inner } });
    }
}
