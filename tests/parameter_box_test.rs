use parameter_box::ParameterBox;

#[test]
fn parameter_box_works() {
    let mut param_box = ParameterBox::new();

    param_box.add::<usize>("param_usize").unwrap();
    param_box.add::<u8>("param_u8").unwrap();
    param_box.add::<u16>("param_u16").unwrap();
    param_box.add::<u32>("param_u32").unwrap();
    param_box.add::<u64>("param_u64").unwrap();
    param_box.add::<u128>("param_u128").unwrap();
    param_box.add::<isize>("param_isize").unwrap();
    param_box.add::<i8>("param_i8").unwrap();
    param_box.add::<i16>("param_i16").unwrap();
    param_box.add::<i32>("param_i32").unwrap();
    param_box.add::<i64>("param_i64").unwrap();
    param_box.add::<i128>("param_i128").unwrap();
    param_box.add::<f32>("param_f32").unwrap();
    param_box.add::<f64>("param_f64").unwrap();
    param_box.add::<String>("param_String").unwrap();
    param_box.add::<isize>("param_None").unwrap();

    param_box.set_value::<usize>("param_usize", 0).unwrap();
    param_box.set_value::<u8>("param_u8", 1).unwrap();
    param_box.set_value::<u16>("param_u16", 23).unwrap();
    param_box.set_value::<u32>("param_u32", 4451).unwrap();
    param_box.set_value::<u64>("param_u64", 12).unwrap();
    param_box.set_value::<u128>("param_u128", 112).unwrap();
    param_box.set_value::<isize>("param_isize", -3).unwrap();
    param_box.set_value::<i8>("param_i8", -9).unwrap();
    param_box.set_value::<i16>("param_i16", -111).unwrap();
    param_box.set_value::<i32>("param_i32", -43).unwrap();
    param_box.set_value::<i64>("param_i64", -4).unwrap();
    param_box.set_value::<i128>("param_i128", -40).unwrap();
    param_box.set_value::<f32>("param_f32", 3.14).unwrap();
    param_box.set_value::<f64>("param_f64", 5.66).unwrap();
    param_box
        .set_value::<String>("param_String", "Apple".to_string())
        .unwrap();

    param_box
        .set_explanation("param_usize", "usize parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_u8", "u8 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_u16", "u16 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_u32", "u32 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_u64", "u64 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_isize", "isize parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_i8", "i8 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_i16", "i16 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_i32", "i32 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_i64", "i64 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_f32", "f32 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_f64", "f64 parameter".to_string())
        .unwrap();
    param_box
        .set_explanation("param_String", "String parameter".to_string())
        .unwrap();

    assert_eq!(
        param_box
            .clone_value::<usize>("param_usize")
            .unwrap()
            .unwrap(),
        0_usize
    );
    assert_eq!(
        param_box.clone_value::<u8>("param_u8").unwrap().unwrap(),
        1_u8
    );
    assert_eq!(
        param_box.clone_value::<u16>("param_u16").unwrap().unwrap(),
        23_u16
    );
    assert_eq!(
        param_box.clone_value::<u32>("param_u32").unwrap().unwrap(),
        4451_u32
    );
    assert_eq!(
        param_box.clone_value::<u64>("param_u64").unwrap().unwrap(),
        12_u64
    );
    assert_eq!(
        param_box
            .clone_value::<u128>("param_u128")
            .unwrap()
            .unwrap(),
        112_u128
    );
    assert_eq!(
        param_box
            .clone_value::<isize>("param_isize")
            .unwrap()
            .unwrap(),
        -3_isize
    );
    assert_eq!(
        param_box.clone_value::<i8>("param_i8").unwrap().unwrap(),
        -9_i8
    );
    assert_eq!(
        param_box.clone_value::<i16>("param_i16").unwrap().unwrap(),
        -111_i16
    );
    assert_eq!(
        param_box.clone_value::<i32>("param_i32").unwrap().unwrap(),
        -43_i32
    );
    assert_eq!(
        param_box.clone_value::<i64>("param_i64").unwrap().unwrap(),
        -4_i64
    );
    assert_eq!(
        param_box
            .clone_value::<i128>("param_i128")
            .unwrap()
            .unwrap(),
        -40_i128
    );
    assert_eq!(
        param_box
            .clone_value::<String>("param_String")
            .unwrap()
            .unwrap(),
        "Apple".to_string()
    );
    assert_eq!(param_box.clone_value::<isize>("param_None").unwrap(), None);
}
