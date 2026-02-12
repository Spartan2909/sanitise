#![feature(prelude_import)]
#![deny(clippy::panic)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::{fs, iter::zip, process::ExitCode};
use clap::Parser;
use sanitise::sanitise_string;
struct Args {
    /// The file to process
    file_name: String,
    /// The root of the output file
    output_file_name: String,
}
#[automatically_derived]
impl ::core::fmt::Debug for Args {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field2_finish(
            f,
            "Args",
            "file_name",
            &self.file_name,
            "output_file_name",
            &&self.output_file_name,
        )
    }
}
#[automatically_derived]
#[allow(unused_qualifications, clippy::redundant_locals)]
impl clap::Parser for Args {}
#[allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_braces,
    unused_qualifications,
)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
    clippy::almost_swapped,
    clippy::redundant_locals,
)]
#[automatically_derived]
impl clap::CommandFactory for Args {
    fn command<'b>() -> clap::Command {
        let __clap_app = clap::Command::new("sanity");
        <Self as clap::Args>::augment_args(__clap_app)
    }
    fn command_for_update<'b>() -> clap::Command {
        let __clap_app = clap::Command::new("sanity");
        <Self as clap::Args>::augment_args_for_update(__clap_app)
    }
}
#[allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_braces,
    unused_qualifications,
)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
    clippy::almost_swapped,
    clippy::redundant_locals,
)]
#[automatically_derived]
impl clap::FromArgMatches for Args {
    fn from_arg_matches(
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        Self::from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn from_arg_matches_mut(
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<Self, clap::Error> {
        #![allow(deprecated)]
        let v = Args {
            file_name: __clap_arg_matches
                .remove_one::<String>("file_name")
                .ok_or_else(|| clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "the following required argument was not provided: file_name",
                ))?,
            output_file_name: __clap_arg_matches
                .remove_one::<String>("output_file_name")
                .ok_or_else(|| clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "the following required argument was not provided: output_file_name",
                ))?,
        };
        ::std::result::Result::Ok(v)
    }
    fn update_from_arg_matches(
        &mut self,
        __clap_arg_matches: &clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        self.update_from_arg_matches_mut(&mut __clap_arg_matches.clone())
    }
    fn update_from_arg_matches_mut(
        &mut self,
        __clap_arg_matches: &mut clap::ArgMatches,
    ) -> ::std::result::Result<(), clap::Error> {
        #![allow(deprecated)]
        if __clap_arg_matches.contains_id("file_name") {
            #[allow(non_snake_case)]
            let file_name = &mut self.file_name;
            *file_name = __clap_arg_matches
                .remove_one::<String>("file_name")
                .ok_or_else(|| clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "the following required argument was not provided: file_name",
                ))?;
        }
        if __clap_arg_matches.contains_id("output_file_name") {
            #[allow(non_snake_case)]
            let output_file_name = &mut self.output_file_name;
            *output_file_name = __clap_arg_matches
                .remove_one::<String>("output_file_name")
                .ok_or_else(|| clap::Error::raw(
                    clap::error::ErrorKind::MissingRequiredArgument,
                    "the following required argument was not provided: output_file_name",
                ))?;
        }
        ::std::result::Result::Ok(())
    }
}
#[allow(
    dead_code,
    unreachable_code,
    unused_variables,
    unused_braces,
    unused_qualifications,
)]
#[allow(
    clippy::style,
    clippy::complexity,
    clippy::pedantic,
    clippy::restriction,
    clippy::perf,
    clippy::deprecated,
    clippy::nursery,
    clippy::cargo,
    clippy::suspicious_else_formatting,
    clippy::almost_swapped,
    clippy::redundant_locals,
)]
#[automatically_derived]
impl clap::Args for Args {
    fn group_id() -> Option<clap::Id> {
        Some(clap::Id::from("Args"))
    }
    fn augment_args<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app
                .group(
                    clap::ArgGroup::new("Args")
                        .multiple(true)
                        .args({
                            let members: [clap::Id; 2usize] = [
                                clap::Id::from("file_name"),
                                clap::Id::from("output_file_name"),
                            ];
                            members
                        }),
                );
            let __clap_app = __clap_app
                .arg({
                    #[allow(deprecated)]
                    let arg = clap::Arg::new("file_name")
                        .value_name("FILE_NAME")
                        .required(true && clap::ArgAction::Set.takes_values())
                        .value_parser({
                            use ::clap_builder::builder::impl_prelude::*;
                            let auto = ::clap_builder::builder::_infer_ValueParser_for::<
                                String,
                            >::new();
                            (&&&&&&auto).value_parser()
                        })
                        .action(clap::ArgAction::Set);
                    let arg = arg.help("The file to process").long_help(None);
                    let arg = arg;
                    arg
                });
            let __clap_app = __clap_app
                .arg({
                    #[allow(deprecated)]
                    let arg = clap::Arg::new("output_file_name")
                        .value_name("OUTPUT_FILE_NAME")
                        .required(true && clap::ArgAction::Set.takes_values())
                        .value_parser({
                            use ::clap_builder::builder::impl_prelude::*;
                            let auto = ::clap_builder::builder::_infer_ValueParser_for::<
                                String,
                            >::new();
                            (&&&&&&auto).value_parser()
                        })
                        .action(clap::ArgAction::Set);
                    let arg = arg.help("The root of the output file").long_help(None);
                    let arg = arg;
                    arg
                });
            __clap_app
        }
    }
    fn augment_args_for_update<'b>(__clap_app: clap::Command) -> clap::Command {
        {
            let __clap_app = __clap_app
                .group(
                    clap::ArgGroup::new("Args")
                        .multiple(true)
                        .args({
                            let members: [clap::Id; 2usize] = [
                                clap::Id::from("file_name"),
                                clap::Id::from("output_file_name"),
                            ];
                            members
                        }),
                );
            let __clap_app = __clap_app
                .arg({
                    #[allow(deprecated)]
                    let arg = clap::Arg::new("file_name")
                        .value_name("FILE_NAME")
                        .required(true && clap::ArgAction::Set.takes_values())
                        .value_parser({
                            use ::clap_builder::builder::impl_prelude::*;
                            let auto = ::clap_builder::builder::_infer_ValueParser_for::<
                                String,
                            >::new();
                            (&&&&&&auto).value_parser()
                        })
                        .action(clap::ArgAction::Set);
                    let arg = arg.help("The file to process").long_help(None);
                    let arg = arg.required(false);
                    arg
                });
            let __clap_app = __clap_app
                .arg({
                    #[allow(deprecated)]
                    let arg = clap::Arg::new("output_file_name")
                        .value_name("OUTPUT_FILE_NAME")
                        .required(true && clap::ArgAction::Set.takes_values())
                        .value_parser({
                            use ::clap_builder::builder::impl_prelude::*;
                            let auto = ::clap_builder::builder::_infer_ValueParser_for::<
                                String,
                            >::new();
                            (&&&&&&auto).value_parser()
                        })
                        .action(clap::ArgAction::Set);
                    let arg = arg.help("The root of the output file").long_help(None);
                    let arg = arg.required(false);
                    arg
                });
            __clap_app
        }
    }
}
fn main() -> ExitCode {
    {
        ::std::io::_print(format_args!("Starting...\n"));
    };
    let args = Args::parse();
    {
        ::std::io::_print(format_args!("Getting CSV contents...\n"));
    };
    let file_contents = match fs::read_to_string(args.file_name) {
        Ok(contents) => contents,
        Err(_) => {
            {
                ::std::io::_eprint(format_args!("Failed to read file\n"));
            };
            return ExitCode::FAILURE;
        }
    };
    {
        ::std::io::_print(format_args!("Processing CSV...\n"));
    };
    #[allow(clippy::type_complexity)]
    let result: Vec<((Vec<i64>, Vec<i64>, Vec<bool>), (Vec<i64>, Vec<i64>))> = match {
        mod __sanitise {
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
                        {
                            ::core::panicking::panic_fmt(
                                format_args!("attempted to extract error from \'Delete\'"),
                            );
                        }
                    }
                }
            }
            enum Action {
                AppendValid,
                IncrementInvalid,
            }
            #[automatically_derived]
            #[doc(hidden)]
            unsafe impl ::core::clone::TrivialClone for Action {}
            #[automatically_derived]
            impl ::core::clone::Clone for Action {
                #[inline]
                fn clone(&self) -> Action {
                    *self
                }
            }
            #[automatically_derived]
            impl ::core::marker::Copy for Action {}
            #[automatically_derived]
            impl ::core::marker::StructuralPartialEq for Action {}
            #[automatically_derived]
            impl ::core::cmp::PartialEq for Action {
                #[inline]
                fn eq(&self, other: &Action) -> bool {
                    let __self_discr = ::core::intrinsics::discriminant_value(self);
                    let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                    __self_discr == __arg1_discr
                }
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
                    if *self { Ok(1.0) } else { Ok(0.0) }
                }
                #[inline(always)]
                fn to_int(&self) -> Result<i64, Interrupt> {
                    if *self { Ok(1) } else { Ok(0) }
                }
                #[inline(always)]
                fn to_string(&self) -> Result<String, Interrupt> {
                    Ok(ToString::to_string(self))
                }
            }
            impl SanitiseConversions for f64 {
                #[inline(always)]
                fn to_bool(&self) -> Result<bool, Interrupt> {
                    Ok(*self != 0.0)
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
                    Ok(*self != 0)
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
                        let message = ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("invalid base for float: \'{0}\'", self),
                            )
                        });
                        Err(Interrupt::Error(message))
                    }
                }
                fn to_int(&self) -> Result<i64, Interrupt> {
                    if let Ok(n) = self.parse() {
                        Ok(n)
                    } else {
                        let message = ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("invalid base for int: \'{0}\'", self),
                            )
                        });
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
            mod validate {
                use super::*;
                struct Column_time {
                    output: Vec<i64>,
                }
                impl Column_time {
                    #[inline(always)]
                    fn new() -> Column_time {
                        Column_time {
                            output: ::alloc::vec::Vec::new(),
                        }
                    }
                    fn invalid(&mut self, value: &i64) -> Result<(), Interrupt> {
                        Err(
                            Interrupt::Error(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "invalid value for column \'time\': {0}",
                                            value,
                                        ),
                                    )
                                }),
                            ),
                        )
                    }
                    fn null(&mut self) -> Result<(), Interrupt> {
                        self.output.push(self.output.last().unwrap_or(&0i64).to_owned());
                        Ok(())
                    }
                    #[inline(always)]
                    fn push_valid(&mut self, value: i64) {
                        self.output.push(value);
                    }
                    #[inline(always)]
                    fn push(
                        &mut self,
                        value: &i64,
                        pulse: Option<&i64>,
                        movement: Option<&i64>,
                    ) -> Result<(), Interrupt> {
                        let value = Some(value);
                        self.push_valid((Ok(value.unwrap().to_owned()))?);
                        Ok(())
                    }
                    fn undo(&mut self) {
                        self.output.pop();
                    }
                    fn aggregate(&self, start_index: usize, end_index: usize) -> i64 {
                        self.output[start_index].clone()
                    }
                    #[inline(always)]
                    fn finish(&mut self) -> Result<(), Interrupt> {
                        Ok(())
                    }
                }
                enum Column_pulse_State {
                    Valid,
                    Invalid {
                        missing: usize,
                        valid_streak: Vec<i64>,
                        last_action: Action,
                    },
                }
                struct Column_pulse {
                    output: Vec<i64>,
                    state: Column_pulse_State,
                }
                impl Column_pulse {
                    #[inline(always)]
                    fn new() -> Column_pulse {
                        Column_pulse {
                            output: ::alloc::vec::Vec::new(),
                            state: Column_pulse_State::Valid,
                        }
                    }
                    fn invalid(&mut self, value: &i64) -> Result<(), Interrupt> {
                        if let Column_pulse_State::Invalid {
                            missing,
                            valid_streak,
                            last_action,
                        } = &mut self.state
                        {
                            *missing += 1;
                            if !valid_streak.is_empty() {
                                *missing += valid_streak.len();
                                *valid_streak = ::alloc::vec::Vec::new();
                            }
                            *last_action = Action::IncrementInvalid;
                        } else {
                            self.state = Column_pulse_State::Invalid {
                                missing: 1,
                                valid_streak: ::alloc::vec::Vec::new(),
                                last_action: Action::IncrementInvalid,
                            };
                        }
                        Ok(())
                    }
                    fn null(&mut self) -> Result<(), Interrupt> {
                        self.output.push(self.output.last().unwrap_or(&0i64).to_owned());
                        Ok(())
                    }
                    #[inline(always)]
                    fn push_valid(&mut self, value: i64) {
                        if let Column_pulse_State::Invalid {
                            missing,
                            valid_streak,
                            last_action,
                        } = &mut self.state
                        {
                            valid_streak.push(value);
                            *last_action = Action::AppendValid;
                            if valid_streak.len() >= 3usize {
                                let before_invalid_streak = self
                                    .output
                                    .last()
                                    .unwrap_or(&valid_streak[0]);
                                let after_invalid_streak = &valid_streak[0];
                                let average = (before_invalid_streak + after_invalid_streak)
                                    / &2;
                                for _ in 0..(*missing) {
                                    self.output.push(average.to_owned())
                                }
                                self.output.extend(valid_streak.iter().cloned());
                                self.state = Column_pulse_State::Valid;
                            }
                        } else {
                            self.output.push(value);
                        }
                    }
                    #[inline(always)]
                    fn push(
                        &mut self,
                        value: &i64,
                        time: Option<&i64>,
                        movement: Option<&i64>,
                    ) -> Result<(), Interrupt> {
                        if value > &100i64 {
                            return self.invalid(value);
                        }
                        if value < &40i64 {
                            return self.invalid(value);
                        }
                        let value = Some(value);
                        self.push_valid((Ok(value.unwrap().to_owned()))?);
                        Ok(())
                    }
                    fn undo(&mut self) {
                        if let Column_pulse_State::Invalid {
                            missing,
                            valid_streak,
                            last_action,
                        } = &mut self.state
                        {
                            if *last_action == Action::AppendValid {
                                valid_streak.pop();
                            } else {
                                *missing -= 1;
                                if *missing == 0 {
                                    self.state = Column_pulse_State::Valid;
                                }
                            }
                        } else {
                            self.output.pop();
                        }
                    }
                    fn aggregate(&self, start_index: usize, end_index: usize) -> i64 {
                        self.output[start_index].clone()
                    }
                    #[inline(always)]
                    fn finish(&mut self) -> Result<(), Interrupt> {
                        if let Column_pulse_State::Invalid {
                            missing,
                            valid_streak,
                            last_action,
                        } = &mut self.state
                        {
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
                }
                struct Column_movement {
                    output: Vec<bool>,
                }
                impl Column_movement {
                    #[inline(always)]
                    fn new() -> Column_movement {
                        Column_movement {
                            output: ::alloc::vec::Vec::new(),
                        }
                    }
                    fn invalid(&mut self, value: &i64) -> Result<(), Interrupt> {
                        Err(
                            Interrupt::Error(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "invalid value for column \'movement\': {0}",
                                            value,
                                        ),
                                    )
                                }),
                            ),
                        )
                    }
                    fn null(&mut self) -> Result<(), Interrupt> {
                        Err(Interrupt::Delete)
                    }
                    #[inline(always)]
                    fn push_valid(&mut self, value: bool) {
                        self.output.push(value);
                    }
                    #[inline(always)]
                    fn push(
                        &mut self,
                        value: &i64,
                        time: Option<&i64>,
                        pulse: Option<&i64>,
                    ) -> Result<(), Interrupt> {
                        if ![0i64.to_owned(), 1i64.to_owned()].contains(value) {
                            return self.invalid(value);
                        }
                        let value = Some(value);
                        self.push_valid(
                            (SanitiseConversions::to_bool(
                                &((Ok(value.unwrap().to_owned()))?),
                            ))?,
                        );
                        Ok(())
                    }
                    fn undo(&mut self) {
                        self.output.pop();
                    }
                    fn aggregate(&self, start_index: usize, end_index: usize) -> bool {
                        self.output[start_index].clone()
                    }
                    #[inline(always)]
                    fn finish(&mut self) -> Result<(), Interrupt> {
                        Ok(())
                    }
                }
                pub(super) fn process(
                    file: (&[Option<i64>], &[Option<i64>], &[Option<i64>]),
                ) -> Result<(Vec<i64>, Vec<i64>, Vec<bool>), (String, usize)> {
                    if file.0.is_empty() {
                        return Err(("Empty file".to_string(), 1));
                    }
                    let mut automaton_0 = Column_time::new();
                    let mut automaton_1 = Column_pulse::new();
                    let mut automaton_2 = Column_movement::new();
                    for i in 0..(file.0.len()) {
                        if let Some(tmp) = &(file.0)[i] {
                            if [-1i64.to_owned()].contains(&tmp) {
                                if let Err(interrupt) = automaton_0.null() {
                                    match interrupt {
                                        Interrupt::Delete => {
                                            continue;
                                        }
                                        Interrupt::Error(s) => return Err((s, i + 1)),
                                    }
                                }
                            } else {
                                if let Err(interrupt) = automaton_0
                                    .push(tmp, file.1[i].as_ref(), file.2[i].as_ref())
                                {
                                    match interrupt {
                                        Interrupt::Delete => {
                                            continue;
                                        }
                                        Interrupt::Error(s) => return Err((s, i + 1)),
                                    }
                                }
                            }
                        } else {
                            if let Err(interrupt) = automaton_0.null() {
                                match interrupt {
                                    Interrupt::Delete => {
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        }
                        if let Some(tmp) = &(file.1)[i] {
                            if [-1i64.to_owned()].contains(&tmp) {
                                if let Err(interrupt) = automaton_1.null() {
                                    match interrupt {
                                        Interrupt::Delete => {
                                            automaton_0.undo();
                                            continue;
                                        }
                                        Interrupt::Error(s) => return Err((s, i + 1)),
                                    }
                                }
                            } else {
                                if let Err(interrupt) = automaton_1
                                    .push(tmp, file.0[i].as_ref(), file.2[i].as_ref())
                                {
                                    match interrupt {
                                        Interrupt::Delete => {
                                            automaton_0.undo();
                                            continue;
                                        }
                                        Interrupt::Error(s) => return Err((s, i + 1)),
                                    }
                                }
                            }
                        } else {
                            if let Err(interrupt) = automaton_1.null() {
                                match interrupt {
                                    Interrupt::Delete => {
                                        automaton_0.undo();
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        }
                        if let Some(tmp) = &(file.2)[i] {
                            if [-1i64.to_owned()].contains(&tmp) {
                                if let Err(interrupt) = automaton_2.null() {
                                    match interrupt {
                                        Interrupt::Delete => {
                                            automaton_0.undo();
                                            automaton_1.undo();
                                            continue;
                                        }
                                        Interrupt::Error(s) => return Err((s, i + 1)),
                                    }
                                }
                            } else {
                                if let Err(interrupt) = automaton_2
                                    .push(tmp, file.0[i].as_ref(), file.1[i].as_ref())
                                {
                                    match interrupt {
                                        Interrupt::Delete => {
                                            automaton_0.undo();
                                            automaton_1.undo();
                                            continue;
                                        }
                                        Interrupt::Error(s) => return Err((s, i + 1)),
                                    }
                                }
                            }
                        } else {
                            if let Err(interrupt) = automaton_2.null() {
                                match interrupt {
                                    Interrupt::Delete => {
                                        automaton_0.undo();
                                        automaton_1.undo();
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        }
                    }
                    if let Err(interrupt) = automaton_0.finish() {
                        return Err((interrupt.extract_error(), file.0.len()));
                    }
                    if let Err(interrupt) = automaton_1.finish() {
                        return Err((interrupt.extract_error(), file.0.len()));
                    }
                    if let Err(interrupt) = automaton_2.finish() {
                        return Err((interrupt.extract_error(), file.0.len()));
                    }
                    let result_0 = automaton_0.output;
                    let result_1 = automaton_1.output;
                    let result_2 = automaton_2.output;
                    Ok((result_0, result_1, result_2))
                }
                pub(super) fn parse(
                    file: &[Vec<&str>],
                ) -> Result<
                    (Vec<Option<i64>>, Vec<Option<i64>>, Vec<Option<i64>>),
                    (String, usize),
                > {
                    let mut column_0 = ::alloc::vec::Vec::new();
                    let mut column_1 = ::alloc::vec::Vec::new();
                    let mut column_2 = ::alloc::vec::Vec::new();
                    for (i, line) in file.iter().enumerate() {
                        if line.len() != 3usize {
                            return Err((
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Invalid line length: {0}", line.len()),
                                    )
                                }),
                                i,
                            ));
                        }
                        if line[0usize].is_empty() {
                            column_0.push(None);
                        } else {
                            column_0
                                .push(
                                    match line[0usize].parse() {
                                        Ok(v) => Some(v),
                                        Err(_) => {
                                            return Err((
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("failed to parse {0}", line[0usize]),
                                                    )
                                                }),
                                                i,
                                            ));
                                        }
                                    },
                                );
                        }
                        if line[1usize].is_empty() {
                            column_1.push(None);
                        } else {
                            column_1
                                .push(
                                    match line[1usize].parse() {
                                        Ok(v) => Some(v),
                                        Err(_) => {
                                            return Err((
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("failed to parse {0}", line[1usize]),
                                                    )
                                                }),
                                                i,
                                            ));
                                        }
                                    },
                                );
                        }
                        if line[2usize].is_empty() {
                            column_2.push(None);
                        } else {
                            column_2
                                .push(
                                    match line[2usize].parse() {
                                        Ok(v) => Some(v),
                                        Err(_) => {
                                            return Err((
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("failed to parse {0}", line[2usize]),
                                                    )
                                                }),
                                                i,
                                            ));
                                        }
                                    },
                                );
                        }
                    }
                    Ok((column_0, column_1, column_2))
                }
            }
            use validate::parse as parse_file;
            mod process {
                use super::*;
                struct Column_time {
                    output: Vec<i64>,
                }
                impl Column_time {
                    #[inline(always)]
                    fn new() -> Column_time {
                        Column_time {
                            output: ::alloc::vec::Vec::new(),
                        }
                    }
                    fn invalid(&mut self, value: &i64) -> Result<(), Interrupt> {
                        Err(
                            Interrupt::Error(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "invalid value for column \'time\': {0}",
                                            value,
                                        ),
                                    )
                                }),
                            ),
                        )
                    }
                    fn null(&mut self) -> Result<(), Interrupt> {
                        Err(
                            Interrupt::Error(
                                "unexpected null in column 'time'".to_owned(),
                            ),
                        )
                    }
                    #[inline(always)]
                    fn push_valid(&mut self, value: i64) {
                        self.output.push(value);
                    }
                    #[inline(always)]
                    fn push(
                        &mut self,
                        value: &i64,
                        pulse: Option<&i64>,
                        movement: Option<&bool>,
                    ) -> Result<(), Interrupt> {
                        let value = Some(value);
                        self.push_valid(
                            (Ok(((Ok(value.unwrap().to_owned()))?) / ((Ok(60000i64))?)))?,
                        );
                        Ok(())
                    }
                    fn undo(&mut self) {
                        self.output.pop();
                    }
                    fn aggregate(&self, start_index: usize, end_index: usize) -> i64 {
                        self.output[start_index].clone()
                    }
                    #[inline(always)]
                    fn finish(&mut self) -> Result<(), Interrupt> {
                        Ok(())
                    }
                }
                struct Column_pulse {
                    output: Vec<i64>,
                }
                impl Column_pulse {
                    #[inline(always)]
                    fn new() -> Column_pulse {
                        Column_pulse {
                            output: ::alloc::vec::Vec::new(),
                        }
                    }
                    fn invalid(&mut self, value: &i64) -> Result<(), Interrupt> {
                        Err(
                            Interrupt::Error(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!(
                                            "invalid value for column \'pulse\': {0}",
                                            value,
                                        ),
                                    )
                                }),
                            ),
                        )
                    }
                    fn null(&mut self) -> Result<(), Interrupt> {
                        Err(
                            Interrupt::Error(
                                "unexpected null in column 'pulse'".to_owned(),
                            ),
                        )
                    }
                    #[inline(always)]
                    fn push_valid(&mut self, value: i64) {
                        self.output.push(value);
                    }
                    #[inline(always)]
                    fn push(
                        &mut self,
                        value: &i64,
                        time: Option<&i64>,
                        movement: Option<&bool>,
                    ) -> Result<(), Interrupt> {
                        let value = Some(value);
                        self.push_valid((Ok(value.unwrap().to_owned()))?);
                        Ok(())
                    }
                    fn undo(&mut self) {
                        self.output.pop();
                    }
                    fn aggregate(&self, start_index: usize, end_index: usize) -> i64 {
                        let mut sum = self.output[end_index].clone();
                        for item in &self.output[start_index..end_index] {
                            sum += item.clone();
                        }
                        sum / (end_index - start_index + 1) as i64
                    }
                    #[inline(always)]
                    fn finish(&mut self) -> Result<(), Interrupt> {
                        Ok(())
                    }
                }
                pub(super) fn process(
                    file: (&[Option<i64>], &[Option<i64>], &[Option<bool>]),
                ) -> Result<(Vec<i64>, Vec<i64>), (String, usize)> {
                    if file.0.is_empty() {
                        return Err(("Empty file".to_string(), 1));
                    }
                    let mut automaton_0 = Column_time::new();
                    let mut automaton_1 = Column_pulse::new();
                    for i in 0..(file.0.len()) {
                        if let Some(tmp) = &(file.0)[i] {
                            if let Err(interrupt) = automaton_0
                                .push(tmp, file.1[i].as_ref(), file.2[i].as_ref())
                            {
                                match interrupt {
                                    Interrupt::Delete => {
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        } else {
                            if let Err(interrupt) = automaton_0.null() {
                                match interrupt {
                                    Interrupt::Delete => {
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        }
                        if let Some(tmp) = &(file.1)[i] {
                            if let Err(interrupt) = automaton_1
                                .push(tmp, file.0[i].as_ref(), file.2[i].as_ref())
                            {
                                match interrupt {
                                    Interrupt::Delete => {
                                        automaton_0.undo();
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        } else {
                            if let Err(interrupt) = automaton_1.null() {
                                match interrupt {
                                    Interrupt::Delete => {
                                        automaton_0.undo();
                                        continue;
                                    }
                                    Interrupt::Error(s) => return Err((s, i + 1)),
                                }
                            }
                        }
                    }
                    if let Err(interrupt) = automaton_0.finish() {
                        return Err((interrupt.extract_error(), file.0.len()));
                    }
                    if let Err(interrupt) = automaton_1.finish() {
                        return Err((interrupt.extract_error(), file.0.len()));
                    }
                    let mut new_result_0 = ::alloc::vec::Vec::new();
                    let mut new_result_1 = ::alloc::vec::Vec::new();
                    let mut start_index = 0;
                    let mut run_value = &automaton_0.output[0];
                    for (i, current_value) in automaton_0.output.iter().enumerate() {
                        if current_value != run_value {
                            new_result_0.push(run_value.to_owned());
                            new_result_1.push(automaton_1.aggregate(start_index, i - 1));
                            start_index = i;
                            run_value = current_value;
                        }
                    }
                    let result_0 = new_result_0;
                    let result_1 = new_result_1;
                    Ok((result_0, result_1))
                }
                pub(super) fn parse(
                    file: &[Vec<&str>],
                ) -> Result<
                    (Vec<Option<i64>>, Vec<Option<i64>>, Vec<Option<bool>>),
                    (String, usize),
                > {
                    let mut column_0 = ::alloc::vec::Vec::new();
                    let mut column_1 = ::alloc::vec::Vec::new();
                    let mut column_2 = ::alloc::vec::Vec::new();
                    for (i, line) in file.iter().enumerate() {
                        if line.len() != 3usize {
                            return Err((
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Invalid line length: {0}", line.len()),
                                    )
                                }),
                                i,
                            ));
                        }
                        if line[0usize].is_empty() {
                            column_0.push(None);
                        } else {
                            column_0
                                .push(
                                    match line[0usize].parse() {
                                        Ok(v) => Some(v),
                                        Err(_) => {
                                            return Err((
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("failed to parse {0}", line[0usize]),
                                                    )
                                                }),
                                                i,
                                            ));
                                        }
                                    },
                                );
                        }
                        if line[1usize].is_empty() {
                            column_1.push(None);
                        } else {
                            column_1
                                .push(
                                    match line[1usize].parse() {
                                        Ok(v) => Some(v),
                                        Err(_) => {
                                            return Err((
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("failed to parse {0}", line[1usize]),
                                                    )
                                                }),
                                                i,
                                            ));
                                        }
                                    },
                                );
                        }
                        if line[2usize].is_empty() {
                            column_2.push(None);
                        } else {
                            column_2
                                .push(
                                    match line[2usize].parse() {
                                        Ok(v) => Some(v),
                                        Err(_) => {
                                            return Err((
                                                ::alloc::__export::must_use({
                                                    ::alloc::fmt::format(
                                                        format_args!("failed to parse {0}", line[2usize]),
                                                    )
                                                }),
                                                i,
                                            ));
                                        }
                                    },
                                );
                        }
                    }
                    Ok((column_0, column_1, column_2))
                }
            }
            fn get_files(csv: &str) -> Vec<Vec<Vec<&str>>> {
                let mut lines: Vec<&str> = csv
                    .split('\n')
                    .map(|s| { s.strip_suffix("\r").unwrap_or(s) })
                    .collect();
                if lines.last().is_some_and(|line| line.is_empty()) {
                    lines.pop();
                }
                lines
                    .split(|&line| line == "time,pulse,movement")
                    .map(|file| {
                        file.iter().map(|line| { line.split(',').collect() }).collect()
                    })
                    .collect()
            }
            #[inline(always)]
            fn process(
                file: &[Vec<&str>],
            ) -> Result<
                ((Vec<i64>, Vec<i64>, Vec<bool>), (Vec<i64>, Vec<i64>)),
                (String, usize),
            > {
                let (item_0, item_1, item_2) = parse_file(file)?;
                let (time_0, pulse_0, movement_0) = validate::process((
                    &item_0,
                    &item_1,
                    &item_2,
                ))?;
                let (time_1, pulse_1) = process::process((
                    &Vec::from_iter(time_0.iter().map(|x| Some(x.to_owned()))),
                    &Vec::from_iter(pulse_0.iter().map(|x| Some(x.to_owned()))),
                    &Vec::from_iter(movement_0.iter().map(|x| Some(x.to_owned()))),
                ))?;
                Ok(((time_0, pulse_0, movement_0), (time_1, pulse_1)))
            }
            #[inline(always)]
            pub(super) fn main(
                csv: &str,
            ) -> Result<
                Vec<((Vec<i64>, Vec<i64>, Vec<bool>), (Vec<i64>, Vec<i64>))>,
                (String, usize),
            > {
                let files = get_files(csv);
                if files.len() < 2 {
                    return Err(("found no headers".to_string(), 1));
                }
                let result = files[1..].iter().map(|file| process(file)).collect();
                result
            }
        }
        __sanitise::main(&file_contents)
    } {
        Ok(v) => v,
        Err((message, line)) => {
            {
                ::std::io::_eprint(format_args!("Line {0}: {1}\n", line, message));
            };
            return ExitCode::FAILURE;
        }
    };
    {
        ::std::io::_print(format_args!("Writing to output files...\n"));
    };
    for (i, ((time_millis, pulse_raw, movement), (time_mins, pulse_average))) in result
        .into_iter()
        .enumerate()
    {
        let file_name_base = args.output_file_name.to_owned()
            + &::alloc::__export::must_use({
                ::alloc::fmt::format(format_args!("_{0}", i + 1))
            });
        let file_name_raw = file_name_base.to_owned() + "_raw.csv";
        let file_name_processed = file_name_base + "_processed.csv";
        let mut buf_raw = String::with_capacity(time_millis.len() * 20);
        let mut buf_processed = String::with_capacity(time_millis.len() * 10);
        buf_raw.push_str("time,pulse,movement\n");
        buf_processed.push_str("time,pulse\n");
        for ((time_millis, pulse), movement) in zip(
            zip(time_millis, pulse_raw),
            movement,
        ) {
            buf_raw
                .push_str(
                    &::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("{0},{1},{2}\n", time_millis, pulse, movement),
                        )
                    }),
                );
        }
        for (time_mins, pulse) in zip(time_mins, pulse_average) {
            buf_processed
                .push_str(
                    &::alloc::__export::must_use({
                        ::alloc::fmt::format(format_args!("{0},{1}\n", time_mins, pulse))
                    }),
                );
        }
        let _ = fs::write(file_name_raw, buf_raw);
        let _ = fs::write(file_name_processed, buf_processed);
    }
    {
        ::std::io::_print(format_args!("Done\n"));
    };
    ExitCode::SUCCESS
}
