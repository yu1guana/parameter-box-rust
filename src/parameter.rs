// Copyright (c) 2021 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use core::fmt::Display;
use std::any::{type_name, Any, TypeId};

#[derive(Debug, Clone)]
pub struct ParameterCore<T: PartialOrd + PartialEq + Clone + Display> {
    /// Parameter value.
    pub value: Option<T>,
    /// Parameter range.
    ///
    /// You can choose both open and clse boundary values.
    pub range: (Option<RangeCondition<T>>, Option<RangeCondition<T>>),
    /// Black list or white list of a parameter
    pub list: Option<ListCondition<T>>,
}

#[derive(Debug)]
pub struct Parameter {
    pub parameter_core: Box<dyn Any>,
    pub type_id: TypeId,
    pub type_string: String,
    pub value_string: Option<String>,
    pub range_string: (Option<String>, Option<String>),
    pub list_string: Option<(String, String)>,
    pub explanation: Option<String>,
    pub unvisible: bool,
}

#[derive(Debug, Clone)]
pub enum RangeCondition<T: PartialOrd + PartialEq + Clone> {
    Open(T),
    Close(T),
}

#[derive(Debug, Clone)]
pub enum ListCondition<T: PartialOrd + PartialEq + Clone> {
    Black(Vec<T>),
    White(Vec<T>),
}

#[derive(Debug, Clone)]
pub enum RangeError {
    LessThanMinLimit(String),
    LargerThanMaxLimit(String),
}

#[derive(Debug, Clone)]
pub enum ListError {
    BlacklistViolation(String),
    WhitelistViolation(String),
}

impl<T> ParameterCore<T>
where
    T: PartialOrd + PartialEq + Clone + Display,
{
    pub fn new() -> Self {
        Self {
            value: None,
            range: (None, None),
            list: None,
        }
    }

    pub fn check_min_limit(&self) -> Result<(), RangeError> {
        if let (Some(value), Some(min_limit)) = (&self.value, &self.range.0) {
            match min_limit {
                RangeCondition::Open(open_min_limit) => {
                    if value <= open_min_limit {
                        return Err(RangeError::LessThanMinLimit(
                            ParameterCore::err_msg_bad_range(">", open_min_limit),
                        ));
                    }
                }
                RangeCondition::Close(close_min_limit) => {
                    if value < close_min_limit {
                        return Err(RangeError::LessThanMinLimit(
                            ParameterCore::err_msg_bad_range("≧", close_min_limit),
                        ));
                    }
                }
            }
        }
        return Ok(());
    }

    pub fn check_max_limit(&self) -> Result<(), RangeError> {
        if let (Some(value), Some(max_limit)) = (&self.value, &self.range.1) {
            match max_limit {
                RangeCondition::Open(open_max_limit) => {
                    if open_max_limit <= value {
                        return Err(RangeError::LargerThanMaxLimit(
                            ParameterCore::err_msg_bad_range("<", open_max_limit),
                        ));
                    }
                }
                RangeCondition::Close(close_max_limit) => {
                    if close_max_limit < value {
                        return Err(RangeError::LargerThanMaxLimit(
                            ParameterCore::err_msg_bad_range("≦", close_max_limit),
                        ));
                    }
                }
            }
        }
        return Ok(());
    }

    pub fn check_list_condition(&self) -> Result<(), ListError> {
        if let (Some(value), Some(list)) = (&self.value, &self.list) {
            match list {
                ListCondition::Black(blacklist) => {
                    if blacklist.contains(value) {
                        return Err(ListError::BlacklistViolation(
                            ParameterCore::err_msg_bad_list("not in the list", blacklist),
                        ));
                    }
                }
                ListCondition::White(whitelist) => {
                    if !(whitelist.contains(value)) {
                        return Err(ListError::WhitelistViolation(
                            ParameterCore::err_msg_bad_list("in the list", whitelist),
                        ));
                    }
                }
            }
        }
        return Ok(());
    }

    fn err_msg_bad_range(condition: &str, limit: &T) -> String {
        format!("{} {}", condition, limit)
    }

    fn err_msg_bad_list(condition: &str, list: &Vec<T>) -> String {
        format!(
            "{} [{}]",
            condition,
            list.iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", "),
        )
    }
}

impl Parameter {
    pub fn new<T>() -> Self
    where
        T: 'static + PartialOrd + PartialEq + Clone + Display,
    {
        Self {
            parameter_core: Box::new(ParameterCore::<T>::new()),
            type_id: TypeId::of::<T>(),
            type_string: type_name::<T>().to_string(),
            value_string: None,
            range_string: (None, None),
            list_string: None,
            explanation: None,
            unvisible: false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! expect_range_error {
        (Ok, $err:expr) => {
            match $err {
                Ok(_) => assert!(true, "Ok"),
                Err(RangeError::LessThanMinLimit(_)) => assert!(false, "Lesser"),
                Err(RangeError::LargerThanMaxLimit(_)) => assert!(false, "Larger"),
            }
        };
        (Less, $err:expr) => {
            match $err {
                Ok(_) => assert!(false, "Ok"),
                Err(RangeError::LessThanMinLimit(_)) => assert!(true, "Lesser"),
                Err(RangeError::LargerThanMaxLimit(_)) => assert!(false, "Larger"),
            }
        };
        (Larger, $err:expr) => {
            match $err {
                Ok(_) => assert!(false, "Ok"),
                Err(RangeError::LessThanMinLimit(_)) => assert!(false, "Lesser"),
                Err(RangeError::LargerThanMaxLimit(_)) => assert!(true, "Larger"),
            }
        };
    }

    macro_rules! expect_list_error {
        (Ok, $err:expr) => {
            match $err {
                Ok(_) => assert!(true, "Ok"),
                Err(ListError::BlacklistViolation(_)) => assert!(false, "Blacklist"),
                Err(ListError::WhitelistViolation(_)) => assert!(false, "Whitelist"),
            }
        };
        (Blacklist, $err:expr) => {
            match $err {
                Ok(_) => assert!(false, "Ok"),
                Err(ListError::BlacklistViolation(_)) => assert!(true, "Blacklist"),
                Err(ListError::WhitelistViolation(_)) => assert!(false, "Whitelist"),
            }
        };
        (Whitelist, $err:expr) => {
            match $err {
                Ok(_) => assert!(false, "Ok"),
                Err(ListError::BlacklistViolation(_)) => assert!(false, "Blacklist"),
                Err(ListError::WhitelistViolation(_)) => assert!(true, "Whitelist"),
            }
        };
    }

    macro_rules! parameter_num_works {
        ($type:ty) => {
            let mut p_core = ParameterCore::new();
            let a: $type = 1 as $type;
            p_core.value = Some(a);
            // Ok
            expect_range_error!(Ok, p_core.check_min_limit());
            expect_range_error!(Ok, p_core.check_max_limit());
            expect_list_error!(Ok, p_core.check_list_condition());
            p_core.range = (
                Some(RangeCondition::Open(0 as $type)),
                Some(RangeCondition::Close(1 as $type)),
            );
            expect_range_error!(Ok, p_core.check_min_limit());
            expect_range_error!(Ok, p_core.check_max_limit());
            p_core.range = (
                Some(RangeCondition::Close(1 as $type)),
                Some(RangeCondition::Open(2 as $type)),
            );
            expect_range_error!(Ok, p_core.check_min_limit());
            expect_range_error!(Ok, p_core.check_max_limit());
            p_core.list = Some(ListCondition::Black(vec![
                0 as $type, 2 as $type, 4 as $type, 5 as $type,
            ]));
            expect_list_error!(Ok, p_core.check_list_condition());
            p_core.list = Some(ListCondition::White(vec![
                0 as $type, 1 as $type, 4 as $type, 5 as $type,
            ]));
            expect_list_error!(Ok, p_core.check_list_condition());
            // Err
            p_core.range = (
                Some(RangeCondition::Open(1 as $type)),
                Some(RangeCondition::Open(2 as $type)),
            );
            expect_range_error!(Less, p_core.check_min_limit());
            p_core.range = (
                Some(RangeCondition::Open(0 as $type)),
                Some(RangeCondition::Open(1 as $type)),
            );
            expect_range_error!(Larger, p_core.check_max_limit());
            p_core.list = Some(ListCondition::Black(vec![
                0 as $type, 1 as $type, 4 as $type, 5 as $type,
            ]));
            expect_list_error!(Blacklist, p_core.check_list_condition());
            p_core.list = Some(ListCondition::White(vec![
                0 as $type, 2 as $type, 4 as $type, 5 as $type,
            ]));
            expect_list_error!(Whitelist, p_core.check_list_condition());
        };
    }

    #[test]
    fn parameter_num_works() {
        parameter_num_works!(u8);
        parameter_num_works!(u16);
        parameter_num_works!(u32);
        parameter_num_works!(u64);
        parameter_num_works!(u128);
        parameter_num_works!(i8);
        parameter_num_works!(i16);
        parameter_num_works!(i32);
        parameter_num_works!(i64);
        parameter_num_works!(i128);
        parameter_num_works!(isize);
        parameter_num_works!(usize);
        parameter_num_works!(f32);
        parameter_num_works!(f64);
    }

    #[test]
    fn parameter_bool_works() {
        let mut p_core = ParameterCore::new();
        let a = true;
        p_core.value = Some(a);
        expect_range_error!(Ok, p_core.check_min_limit());
        expect_range_error!(Ok, p_core.check_max_limit());
        expect_list_error!(Ok, p_core.check_list_condition());
        p_core.list = Some(ListCondition::White(vec![true]));
        expect_list_error!(Ok, p_core.check_list_condition());
        p_core.list = Some(ListCondition::White(vec![false]));
        expect_list_error!(Whitelist, p_core.check_list_condition());
    }

    #[test]
    fn parameter_string_works() {
        let mut p_core = ParameterCore::new();
        let a: String = "hello".to_string();
        p_core.value = Some(a);
        // Ok
        expect_range_error!(Ok, p_core.check_min_limit());
        expect_range_error!(Ok, p_core.check_max_limit());
        expect_list_error!(Ok, p_core.check_list_condition());
        p_core.list = Some(ListCondition::Black(vec![
            "good mornig".to_string(),
            "good afternoon".to_string(),
            "good night".to_string(),
        ]));
        expect_list_error!(Ok, p_core.check_list_condition());
        p_core.list = Some(ListCondition::White(vec![
            "hello".to_string(),
            "world".to_string(),
            "!".to_string(),
        ]));
        expect_list_error!(Ok, p_core.check_list_condition());
        // Err
        p_core.list = Some(ListCondition::Black(vec![
            "hello".to_string(),
            "world".to_string(),
            "!".to_string(),
        ]));
        expect_list_error!(Blacklist, p_core.check_list_condition());
        p_core.list = Some(ListCondition::White(vec![
            "good mornig".to_string(),
            "good afternoon".to_string(),
            "good night".to_string(),
        ]));
        expect_list_error!(Whitelist, p_core.check_list_condition());
    }
}
