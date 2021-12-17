// Copyright (c) 2021 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

mod parameter;

use core::fmt::Display;
use core::str::FromStr;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[cfg(debug_assertions)]
use std::panic::Location;

use parameter::{ListCondition, ListError, Parameter, ParameterCore, RangeCondition, RangeError};

#[derive(Debug)]
pub struct ParameterBox {
    parameter_list: HashMap<String, Parameter>,
    added_order: Vec<String>,
    error_counter: u32,
}

#[derive(Debug)]
pub enum ParameterBoxError {
    InvalidCondition(String),
    AlreadyAdded(String),
    NotAdded(String),
    InvalidParse(String),
    InvalidInputFile(String),
    IoError(String),
}

#[cfg(debug_assertions)]
macro_rules! err_msg_header {
    () => {
        format!("ParameterBoxError ({}):", Location::caller())
    };
}

#[cfg(not(debug_assertions))]
macro_rules! err_msg_header {
    () => {
        "ParameterBoxError:"
    };
}

macro_rules! err_msg_already_added {
    ($name:expr) => {
        format!(
            "{0} `{1}` has already been added to a parameter box.",
            err_msg_header!(),
            $name
        )
    };
}

macro_rules! err_msg_not_added {
    ($name:expr) => {
        format!(
            "{0} `{1}` has not been added to a parameter box.",
            err_msg_header!(),
            $name
        )
    };
}

macro_rules! err_msg_bad_condition {
    ($name:expr, $value:expr,$condition:expr) => {
        &format!(
            "{0} `{1}` = {2} does not satisfy the condtion that `{1}` {3}.",
            err_msg_header!(),
            $name,
            $value,
            $condition,
        )
    };
}

#[macro_export]
macro_rules! unwrap_result{
    ($parameter_box:ident . $func:ident :: <$type:ty> ( $($args:expr),* ))=>{
        match $parameter_box.$func::<$type>($($args),*){
            Ok(x) => x,
            Err(ParameterBoxError::InvalidCondition(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::AlreadyAdded(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::NotAdded(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::InvalidParse(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::InvalidInputFile(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::IoError(io_error)) => {
                eprintln!("{}", io_error);
                std::process::exit(1);
            },
        }
    };
    ($parameter_box:ident . $func:ident ( $($args:expr),* ))=>{
        match $parameter_box.$func($($args),*){
            Ok(x) => x,
            Err(ParameterBoxError::InvalidCondition(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::AlreadyAdded(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::NotAdded(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::InvalidParse(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::InvalidInputFile(err_msg)) => {
                eprintln!("{}", err_msg);
                std::process::exit(1);
            },
            Err(ParameterBoxError::IoError(io_error)) => {
                eprintln!("{}", io_error);
                std::process::exit(1);
            },
        }
    };
}

impl ParameterBox {
    pub fn new() -> Self {
        Self {
            parameter_list: HashMap::<String, Parameter>::new(),
            added_order: Vec::<String>::new(),
            error_counter: 0,
        }
    }

    #[track_caller]
    pub fn add<T>(&mut self, name: &str) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if self.parameter_list.contains_key(name) {
            self.error_counter += 1;
            Err(ParameterBoxError::AlreadyAdded(err_msg_already_added!(
                name
            )))
        } else {
            let name_string = name.to_string();
            self.parameter_list
                .insert(name_string.clone(), Parameter::new::<T>());
            self.added_order.push(name_string);
            Ok(())
        }
    }

    #[track_caller]
    pub fn set_value<T>(&mut self, name: &str, value: T) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            let mut error_sequence = false;
            let mut err_msg = String::new();
            let mut new_parameter_core = Box::new(
                parameter
                    .parameter_core
                    .as_ref()
                    .downcast_ref::<ParameterCore<T>>()
                    .expect("Downcast failed.")
                    .clone(),
            );
            new_parameter_core.value = Some(value);
            let value = new_parameter_core.value.as_ref().unwrap();
            if let Err(RangeError::LessThanMinLimit(condition)) =
                new_parameter_core.check_min_limit()
            {
                self.error_counter += 1;
                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                err_msg.push_str(err_msg_bad_condition!(name, value, condition));
            }
            if let Err(RangeError::LargerThanMaxLimit(condition)) =
                new_parameter_core.check_max_limit()
            {
                self.error_counter += 1;
                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                err_msg.push_str(err_msg_bad_condition!(name, value, condition));
            }
            match new_parameter_core.check_list_condition() {
                Err(ListError::BlacklistViolation(condition)) => {
                    self.error_counter += 1;
                    ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                    err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                }
                Err(ListError::WhitelistViolation(condition)) => {
                    self.error_counter += 1;
                    ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                    err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                }
                Ok(()) => (),
            }
            parameter.value_string = Some(format!("{}", value));
            parameter.parameter_core = new_parameter_core;
            if err_msg.is_empty() {
                Ok(())
            } else {
                Err(ParameterBoxError::InvalidCondition(err_msg))
            }
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    pub fn set_range_open_open<T>(
        &mut self,
        name: &str,
        range: (T, T),
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_range(
            name,
            (RangeCondition::Open(range.0), RangeCondition::Open(range.1)),
        )
    }

    #[track_caller]
    pub fn set_range_open_close<T>(
        &mut self,
        name: &str,
        range: (T, T),
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_range(
            name,
            (
                RangeCondition::Open(range.0),
                RangeCondition::Close(range.1),
            ),
        )
    }

    #[track_caller]
    pub fn set_range_close_open<T>(
        &mut self,
        name: &str,
        range: (T, T),
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_range(
            name,
            (
                RangeCondition::Close(range.0),
                RangeCondition::Open(range.1),
            ),
        )
    }

    #[track_caller]
    pub fn set_range_close_close<T>(
        &mut self,
        name: &str,
        range: (T, T),
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_range(
            name,
            (
                RangeCondition::Close(range.0),
                RangeCondition::Close(range.1),
            ),
        )
    }

    #[track_caller]
    pub fn set_min_limit_open<T>(
        &mut self,
        name: &str,
        min_limit: T,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_min_limit(name, RangeCondition::Open(min_limit))
    }

    #[track_caller]
    pub fn set_min_limit_close<T>(
        &mut self,
        name: &str,
        min_limit: T,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_min_limit(name, RangeCondition::Close(min_limit))
    }

    #[track_caller]
    pub fn set_max_limit_open<T>(
        &mut self,
        name: &str,
        max_limit: T,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_max_limit(name, RangeCondition::Open(max_limit))
    }

    #[track_caller]
    pub fn set_max_limit_close<T>(
        &mut self,
        name: &str,
        max_limit: T,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_max_limit(name, RangeCondition::Close(max_limit))
    }

    #[track_caller]
    pub fn set_blacklist<T>(
        &mut self,
        name: &str,
        blacklist: Vec<T>,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_list_info(name, ListCondition::Black(blacklist))
    }

    #[track_caller]
    pub fn set_whitelist<T>(
        &mut self,
        name: &str,
        whitelist: Vec<T>,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        self.set_list_info(name, ListCondition::White(whitelist))
    }

    #[track_caller]
    pub fn set_explanation(
        &mut self,
        name: &str,
        explanation: String,
    ) -> Result<(), ParameterBoxError> {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            parameter.explanation = Some(explanation);
            Ok(())
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    pub fn set_unvisible(&mut self, name: &str) -> Result<(), ParameterBoxError> {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            parameter.unvisible = true;
            Ok(())
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    pub fn ref_value<T>(&mut self, name: &str) -> Result<&Option<T>, ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get(name) {
            Ok(&(parameter
                .parameter_core
                .as_ref()
                .downcast_ref::<ParameterCore<T>>()
                .expect("Downcast failed.")
                .value))
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    pub fn clone_value<T>(&mut self, name: &str) -> Result<Option<T>, ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get(name) {
            Ok(parameter
                .parameter_core
                .as_ref()
                .downcast_ref::<ParameterCore<T>>()
                .expect("Downcast failed.")
                .value
                .clone())
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    pub fn clone_value_forcibly<T>(&mut self, name: &str) -> T
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        match unwrap_result!(self.clone_value::<T>(name)) {
            Some(value) => value,
            None => {
                eprintln!("{} `{}` does not have a value.", err_msg_header!(), name);
                std::process::exit(1);
            }
        }
    }

    #[track_caller]
    pub fn ref_explanation(&mut self, name: &str) -> Result<&Option<String>, ParameterBoxError> {
        if let Some(parameter) = self.parameter_list.get(name) {
            Ok(&parameter.explanation)
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    pub fn get_num_errors(&self) -> &u32 {
        &self.error_counter
    }

    #[track_caller]
    pub fn read_file(&mut self, filename: &str) -> Result<(), ParameterBoxError> {
        let file;
        match File::open(filename) {
            Ok(opened_file) => file = opened_file,
            Err(err) => return Err(err.into()),
        }
        let mut duplicate_checker: HashMap<String, Vec<u32>> = HashMap::new();
        let mut line_number = 0_u32;
        let mut error_sequence = false;
        let mut err_msg = String::new();
        let comment_line_header = "#";
        for line_content in BufReader::new(file).lines() {
            line_number += 1;
            let line;
            match line_content {
                Ok(unwrapped_line) => line = unwrapped_line,
                Err(err) => return Err(err.into()),
            }
            if line.starts_with(comment_line_header) {
                continue;
            }
            if line.is_empty() {
                continue;
            }
            let name_value: Vec<&str> = line.split_whitespace().collect();
            let name = name_value[0];
            if !(self.parameter_list.contains_key(name)) {
                self.error_counter += 1;
                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                err_msg.push_str(&format!(
                    "{} in the {}-th line of the file '{}', `{}` has not been added to a parameter box.",
                    err_msg_header!(),
                    line_number,
                    filename,
                    name
                ));
                continue;
            }
            if let Some(checker_element) = duplicate_checker.get_mut(name) {
                checker_element.push(line_number);
            } else {
                duplicate_checker.insert(name.to_string(), vec![line_number]);
            }
            if name_value.len() != 2 {
                self.error_counter += 1;
                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                err_msg.push_str(&format!( "{} in the {}-th line of the file '{}', each line must be '<name> <value>' in a parameter file.",
                        err_msg_header!(), line_number, filename,
                ));
                continue;
            }
            let value_string = name_value[1];
            let mut type_error = true;
            macro_rules! set_correct_value_by_string {
                ($type:ty) => {
                    if self.parameter_list[name].type_id == std::any::TypeId::of::<$type>() {
                        type_error = false;
                        match self.set_value_by_string::<$type>(name, value_string){
                            Ok(()) => (),
                            Err(ParameterBoxError::InvalidCondition(msg)) => {
                                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                                err_msg.push_str(&format!("{} (in the {}-th line of the file '{}')",msg,line_number,filename));
                            },
                            Err(_) => unreachable!(),
                        }
                    }
                };
                ($type_head:ty, $($type_tail:ty),+ ) =>{
                    set_correct_value_by_string!($type_head);
                    set_correct_value_by_string!($($type_tail),+);
                };
            }
            set_correct_value_by_string!(
                bool, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64, String
            );
            if type_error {
                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                err_msg.push_str(&format!(
                    "{} in the {}-th line of the file '{}', the type of `{}` is {}, which cannot be read from files.",
                    err_msg_header!(),
                    line_number,
                    filename,
                    name,
                    self.parameter_list[name].type_string
                ));
            }
        }
        for (name, line_number_list) in duplicate_checker {
            if line_number_list.len() != 1 {
                ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                err_msg.push_str(&format!(
                    "{} in the {} lines of the file '{}', `{}` is duplicate.",
                    err_msg_header!(),
                    line_number_list
                        .iter()
                        .map(|x| format!("{}-th", x))
                        .collect::<Vec<String>>()
                        .join(", "),
                    filename,
                    name,
                ));
            }
        }
        if err_msg.is_empty() {
            Ok(())
        } else {
            Err(ParameterBoxError::InvalidInputFile(err_msg))
        }
    }

    #[track_caller]
    pub fn print<T: Write>(&self, writer: &mut T) -> Result<(), ParameterBoxError> {
        match self.print_core(writer) {
            Ok(()) => Ok(()),
            Err(io_error) => Err(io_error.into()),
        }
    }

    #[track_caller]
    fn set_value_by_string<T>(
        &mut self,
        name: &str,
        value_string: &str,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display + FromStr,
    {
        match T::from_str(value_string) {
            Ok(value) => self.set_value(name, value),
            Err(_) => Err(ParameterBoxError::InvalidParse(format!(
                "{} cannot parse to {}.",
                err_msg_header!(),
                std::any::type_name::<T>(),
            ))),
        }
    }

    #[track_caller]
    fn set_range<T>(
        &mut self,
        name: &str,
        range: (RangeCondition<T>, RangeCondition<T>),
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            let mut error_sequence = false;
            let mut err_msg = String::new();
            let mut new_parameter_core = Box::new(
                parameter
                    .parameter_core
                    .as_ref()
                    .downcast_ref::<ParameterCore<T>>()
                    .expect("Downcast failed.")
                    .clone(),
            );
            parameter.range_string.0 = ParameterBox::make_min_limit_string(&range.0);
            parameter.range_string.1 = ParameterBox::make_max_limit_string(&range.1);
            new_parameter_core.range = (Some(range.0), Some(range.1));
            if let Some(value) = &new_parameter_core.value {
                if let Err(RangeError::LessThanMinLimit(condition)) =
                    new_parameter_core.check_min_limit()
                {
                    self.error_counter += 1;
                    ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                    err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                }
                if let Err(RangeError::LargerThanMaxLimit(condition)) =
                    new_parameter_core.check_max_limit()
                {
                    self.error_counter += 1;
                    ParameterBox::sequence_err_or_not(&mut error_sequence, &mut err_msg);
                    err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                }
            }
            parameter.parameter_core = new_parameter_core;
            if err_msg.is_empty() {
                Ok(())
            } else {
                Err(ParameterBoxError::InvalidCondition(err_msg))
            }
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    fn set_min_limit<T>(
        &mut self,
        name: &str,
        min_limit: RangeCondition<T>,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            let mut err_msg = String::new();
            let mut new_parameter_core = Box::new(
                parameter
                    .parameter_core
                    .as_ref()
                    .downcast_ref::<ParameterCore<T>>()
                    .expect("Downcast failed.")
                    .clone(),
            );
            parameter.range_string.0 = ParameterBox::make_min_limit_string(&min_limit);
            new_parameter_core.range.0 = Some(min_limit);
            if let Some(value) = &new_parameter_core.value {
                if let Err(RangeError::LessThanMinLimit(condition)) =
                    new_parameter_core.check_min_limit()
                {
                    self.error_counter += 1;
                    err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                }
            }
            parameter.parameter_core = new_parameter_core;
            if err_msg.is_empty() {
                Ok(())
            } else {
                Err(ParameterBoxError::InvalidCondition(err_msg))
            }
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    fn set_max_limit<T>(
        &mut self,
        name: &str,
        max_limit: RangeCondition<T>,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            let mut err_msg = String::new();
            let mut new_parameter_core = Box::new(
                parameter
                    .parameter_core
                    .as_ref()
                    .downcast_ref::<ParameterCore<T>>()
                    .expect("Downcast failed.")
                    .clone(),
            );
            parameter.range_string.1 = ParameterBox::make_max_limit_string(&max_limit);
            new_parameter_core.range.1 = Some(max_limit);
            if let Some(value) = &new_parameter_core.value {
                if let Err(RangeError::LargerThanMaxLimit(condition)) =
                    new_parameter_core.check_max_limit()
                {
                    self.error_counter += 1;
                    err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                }
            }
            parameter.parameter_core = new_parameter_core;
            if err_msg.is_empty() {
                Ok(())
            } else {
                Err(ParameterBoxError::InvalidCondition(err_msg))
            }
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    #[track_caller]
    fn set_list_info<T>(
        &mut self,
        name: &str,
        list: ListCondition<T>,
    ) -> Result<(), ParameterBoxError>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        if let Some(parameter) = self.parameter_list.get_mut(name) {
            let mut err_msg = String::new();
            let mut new_parameter_core = Box::new(
                parameter
                    .parameter_core
                    .as_ref()
                    .downcast_ref::<ParameterCore<T>>()
                    .expect("Downcast failed.")
                    .clone(),
            );
            parameter.list_string = ParameterBox::make_list_info_string(&list);
            new_parameter_core.list = Some(list);
            if let Some(value) = &new_parameter_core.value {
                match new_parameter_core.check_list_condition() {
                    Err(ListError::BlacklistViolation(condition)) => {
                        self.error_counter += 1;
                        err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                    }
                    Err(ListError::WhitelistViolation(condition)) => {
                        self.error_counter += 1;
                        err_msg.push_str(err_msg_bad_condition!(name, value, condition));
                    }
                    Ok(()) => (),
                }
            }
            parameter.parameter_core = new_parameter_core;
            if err_msg.is_empty() {
                Ok(())
            } else {
                Err(ParameterBoxError::InvalidCondition(err_msg))
            }
        } else {
            self.error_counter += 1;
            Err(ParameterBoxError::NotAdded(err_msg_not_added!(name)))
        }
    }

    fn print_core<T: Write>(&self, writer: &mut T) -> std::io::Result<()> {
        let mut writer = BufWriter::new(writer);
        for name in self.added_order.iter() {
            let parameter = self.parameter_list.get(name).unwrap();
            if parameter.unvisible {
                continue;
            }
            // Name
            writeln!(writer, "{}\n----------------------------", name)?;
            // Type
            writeln!(writer, "{:14}| {}", "Type", &parameter.type_string)?;
            // Value
            if let Some(value_string) = &parameter.value_string {
                writeln!(writer, "{:14}| {}", "Default value", value_string)?;
            }
            // Range
            match &parameter.range_string {
                (Some(min_limit_string), Some(max_limit_string)) => {
                    writeln!(
                        writer,
                        "{:14}| {} {} {}",
                        "Range", min_limit_string, name, max_limit_string
                    )?;
                }
                (Some(min_limit_string), None) => {
                    writeln!(writer, "{:14}| {} {}", "Range", min_limit_string, name)?;
                }
                (None, Some(max_limit_string)) => {
                    writeln!(writer, "{:14}| {} {}", "Range", name, max_limit_string)?;
                }
                (None, None) => (),
            }
            // List
            if let Some(list_string) = &parameter.list_string {
                writeln!(writer, "{:14}| {}", &list_string.0, &list_string.1)?;
            }
            // Explanation
            if let Some(explanation) = &parameter.explanation {
                writeln!(writer, "{:14}| {}", "Explanation", explanation)?;
            }
            writeln!(writer, "")?;
        }
        writer.flush()
    }

    fn make_min_limit_string<T>(min_limit: &RangeCondition<T>) -> Option<String>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        match min_limit {
            RangeCondition::Open(min_limit) => Some(format!("{} <", min_limit)),
            RangeCondition::Close(min_limit) => Some(format!("{} ≦", min_limit)),
        }
    }

    fn make_max_limit_string<T>(max_limit: &RangeCondition<T>) -> Option<String>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        match max_limit {
            RangeCondition::Open(max_limit) => Some(format!("< {}", max_limit)),
            RangeCondition::Close(max_limit) => Some(format!("≦ {}", max_limit)),
        }
    }

    fn make_list_info_string<T>(list: &ListCondition<T>) -> Option<(String, String)>
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        match list {
            ListCondition::Black(blacklist) => Some((
                "Blacklist".to_string(),
                format!(
                    "[{}]",
                    blacklist
                        .iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join(", "),
                ),
            )),
            ListCondition::White(blacklist) => Some((
                "Whitelist".to_string(),
                format!(
                    "Whitelist: [{}]",
                    blacklist
                        .iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join(", "),
                ),
            )),
        }
    }

    fn sequence_err_or_not(error_sequence: &mut bool, err_msg: &mut String) {
        if *error_sequence {
            err_msg.push('\n');
        } else {
            *error_sequence = true;
        };
    }
}

impl From<std::io::Error> for ParameterBoxError {
    #[track_caller]
    fn from(err: std::io::Error) -> ParameterBoxError {
        ParameterBoxError::IoError(format!("{} {}", err_msg_header!(), err))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn set_range_works() {
        let mut param_box = ParameterBox::new();
        param_box.add::<i32>("p1").unwrap();
        match param_box.set_range("p1", (RangeCondition::Open(1), RangeCondition::Open(10))) {
            Err(_) => assert!(false),
            Ok(_) => assert!(true),
        }
    }
}
