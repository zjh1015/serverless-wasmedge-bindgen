use wasmedge_tensorflow_interface;
use bytecodec::DecodeExt;
use httpcodec::{
    DecodeOptions, HttpVersion, ReasonPhrase, Request, RequestDecoder, Response, StatusCode,
};

use std::io::{Read, Write};
#[cfg(feature = "std")]
use std::net::{Shutdown, TcpListener, TcpStream};
#[cfg(not(feature = "std"))]
use wasmedge_wasi_socket::{Shutdown, TcpListener, TcpStream};

fn classify(image: &[u8]) -> String {
 
    let model_data: &[u8] =include_bytes!("models/mobilenet/mobilenet_v2.tflite");
    let labels = include_str!("models/mobilenet/mobilenet_v2.txt");

    let flat_img = wasmedge_tensorflow_interface::load_jpg_image_to_rgb32f(image, 224, 224);

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
        format!("在这张图片中没有发现花")
    }
}

fn handle_http(req: Request<Vec<u8>>) -> bytecodec::Result<Response<String>> {
    let image = classify(req.body());
    //let res = format!("{}=> {:?}", req.body().len(), image.len());
    // let res = base64::encode(&image);
    //let res = req.body().len();
    println!("{:?}", image);
    Ok(Response::new(
        HttpVersion::V1_0,
        StatusCode::new(200)?,
        ReasonPhrase::new("")?,
        image,
    ))
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buff = [0u8; 1024];
    let mut data = Vec::new();

    loop {
        let n = stream.read(&mut buff)?;
        data.extend_from_slice(&buff[0..n]);
        if n < 1024 {
            break;
        }
    }

    let body_decoder = httpcodec::BodyDecoder::<bytecodec::bytes::RemainingBytesDecoder>::default();

    // According to https://github.com/sile/httpcodec/blob/master/src/message.rs#L30
    // For processing large image, set this option for enlarging the max_bytes
    // There is a bug in httpcodec, it will not process large image correctly
    let option = DecodeOptions {
        max_start_line_size: 0xFFFF,
        max_header_size: 0xFFFF,
    };
    let mut decoder = RequestDecoder::<
        httpcodec::BodyDecoder<bytecodec::bytes::RemainingBytesDecoder>,
    >::with_options(body_decoder, option);

    let req = match decoder.decode_from_bytes(data.as_slice()) {
        Ok(req) => handle_http(req),
        Err(e) => Err(e),
    };
    
    let r = match req {
        Ok(r) => r,
        Err(e) => {
            let err = format!("{:?}", e);
            Response::new(
                HttpVersion::V1_1,
                StatusCode::new(500).unwrap(),
                ReasonPhrase::new(err.as_str()).unwrap(),
                err.clone(),
            )
        }
    };

    let write_buf = r.to_string();
    stream.write(write_buf.as_bytes())?;
    stream.shutdown(Shutdown::Both)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT").unwrap_or(9005.to_string());
    println!("new connection at {}", port);
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    loop {
        let _ = handle_client(listener.accept()?.0);
    }
}
