use wasmedge_tensorflow_interface;

pub fn infer_internal(image_data: &[u8]) -> String {
    let model_data: &[u8] =include_bytes!("models/mobilenet/mobilenet_v2.tflite");
    let labels = include_str!("models/mobilenet/mobilenet_v2.txt");

    let flat_img = wasmedge_tensorflow_interface::load_jpg_image_to_rgb32f(image_data, 224, 224);

    let mut session = wasmedge_tensorflow_interface::Session::new(
        &model_data,
        wasmedge_tensorflow_interface::ModelType::TensorFlowLite,
    );
    session
        .add_input("functional_1_input", &flat_img, &[1, 224, 224, 3])
        .run();
    let res_vec: Vec<f32> = session.get_output("Identity");

    let mut i = 0;
    let mut max_index: i32 = -1;
    let mut max_value: f32 = 0.0;
    while i < res_vec.len() {
        let cur = res_vec[i];
        if cur > max_value {
            max_value = cur;
            max_index = i as i32;
        }
        i += 1;
        println!("{} : {}", i, cur);
    }
    println!("{} : {}", max_index, max_value);
    let mut confidence = "也许有";
    if max_value > 0.8 {
        confidence = "很可能有";
    } else if max_value > 0.6 {
        confidence = "可能有";
    } else if max_value > 0.3 {
        confidence = "也许有";
    }

    let mut label_lines = labels.lines();
    for _i in 0..max_index {
        label_lines.next();
    }

    let class_name = label_lines.next().unwrap().to_string();
    if max_value > 0.3 {
        format!(
            "刚刚上传的图片里面{}<a href='https://www.baidu.com/s?ie=utf-8&wd={}'>{}</a>",
            confidence.to_string(),
            class_name,
            class_name
        )
    } else {
        format!("It does not appears to be any food item in the picture.")
    }
}
