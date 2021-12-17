use parameter_box::{ParameterBox, ParameterBoxError};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2_usize {
        eprintln!("Usage: {} <parameter file>", args[0]);
        std::process::exit(1);
    }

    let mut parameter_box = ParameterBox::new();

    parameter_box::unwrap_result!(parameter_box.add::<i32>("a"));
    parameter_box::unwrap_result!(parameter_box.add::<String>("b"));
    parameter_box::unwrap_result!(parameter_box.add::<f64>("c"));
    parameter_box::unwrap_result!(parameter_box.add::<String>("d"));

    parameter_box::unwrap_result!(parameter_box.set_range_close_close::<i32>("a", (-10, 10)));

    parameter_box::unwrap_result!(parameter_box.set_blacklist::<i32>("a", vec![2, 3, 4]));

    parameter_box::unwrap_result!(parameter_box.read_file(&args[1]));

    let a = parameter_box.clone_value_forcibly::<i32>("a");
    let b = parameter_box.clone_value_forcibly::<String>("b");
    let c = parameter_box.clone_value_forcibly::<f64>("c");

    println!("{} {} {}", a, b, c);
}
