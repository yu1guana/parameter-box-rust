# ParameterBox

This is the Rust library for reading parameters from files.

## Example

The below example shows reading from the file 'paramer.txt'

```rs
use parameter_box::{ParameterBox, ParameterBoxError};

fn main() {
    let mut parameter_box = ParameterBox::new();

    parameter_box::unwrap_result!(parameter_box.add::<i32>("a"));
    parameter_box::unwrap_result!(parameter_box.add::<String>("b"));
    parameter_box::unwrap_result!(parameter_box.add::<f64>("c"));

    parameter_box::unwrap_result!(parameter_box.read_file("parameter.txt"));

    let a = parameter_box.clone_value_forcibly::<i32>("a");
    let b = parameter_box.clone_value_forcibly::<String>("b");
    let c = parameter_box.clone_value_forcibly::<f64>("c");

    println!("{} {} {}", a, b, c);
}
```

The file 'parameter.txt' is as follows.

```
# This is a comment line
a 1
b 2
c 3
```

## License
Copyright (c) 2021 Yuichi Ishida  
Released under the MIT license  
[https://opensource.org/licenses/mit-license.php](https://opensource.org/licenses/mit-license.php)
