use parameter_box::{unwrap_result, ParameterBox, ParameterBoxError};

fn main() {
    let mut param_box = ParameterBox::new();
    unwrap_result!(param_box.add::<u8>("param_u8"));
    unwrap_result!(param_box.add::<i32>("param_i32"));
    unwrap_result!(param_box.add::<i64>("param_i64"));
    unwrap_result!(param_box.add::<f64>("param_f64"));
    unwrap_result!(param_box.add::<String>("param_String"));

    unwrap_result!(param_box.set_unvisible("param_i64"));

    unwrap_result!(param_box.set_value::<u8>("param_u8", 3));
    unwrap_result!(param_box.set_value::<i64>("param_i64", 98));

    unwrap_result!(param_box.set_max_limit_close::<u8>("param_u8", 20));

    unwrap_result!(param_box.set_blacklist::<i32>("param_i32", vec![-3, -1, 2, 4, 6]));

    unwrap_result!(param_box.set_range_close_close::<i64>("param_i64", (10, 100)));

    unwrap_result!(param_box.set_range_open_close::<f64>("param_f64", (-2.0, 3.0)));
    unwrap_result!(param_box.set_explanation("param_u8", "This is a u8 parameter.".to_string()));
    unwrap_result!(param_box.set_explanation("param_i32", "This is a i32 parameter.".to_string()));
    unwrap_result!(param_box.set_explanation("param_f64", "This is a f64 parameter.".to_string()));
    unwrap_result!(
        param_box.set_explanation("param_String", "This is a String parameter.".to_string())
    );

    unwrap_result!(param_box.print(&mut std::io::stdout()));
}
