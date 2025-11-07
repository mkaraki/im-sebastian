use std::ffi::c_int;
use std::io::{Cursor, Write};
use image::{DynamicImage, EncodableLayout};
use crate::image_bridge::register_image_types;
use cfg_if::cfg_if;
use opentelemetry::global::tracer;
use opentelemetry::trace::Tracer;

mod image_bridge;
mod raw_image_decoder;
pub mod structs;
mod reader;
mod resizer;

pub use structs::*;

pub fn read_and_convert(src_param: SourceParam, convert_param: ConvertParam) -> Result<ServedImage, ()> {
    register_image_types();
    let tracer = tracer("im-sebastian::thumb");

    let src_image = tracer.in_span("img.read", |_| {
        reader::read_img(src_param)
    });
    if src_image.is_err() {
        return Err(());
    }
    let mut src_image = src_image.unwrap();

    if convert_param.resize_param.is_some() {
        let src_image_try: Result<DynamicImage, ()> = tracer.in_span("img.resize", |_| {
            resizer::resize(&src_image, &convert_param.resize_param.unwrap())
        });
        if src_image_try.is_err() {
            return Err(());
        }
        src_image = src_image_try?;
    }
    let src_image = src_image;


    let result: Result<ServedImage, ()> = tracer.in_span("img.export", |_| {
        let mut result = ServedImage::default();
        let mut cursor = Cursor::new(&mut result.content);

        match convert_param.export_format {
            ImageBinaryFormat::Webp => {
                let webp_param = convert_param.webp_export_param.unwrap();
                result.content_extension = "webp".to_string();
                result.content_type = "image/webp".to_string();
                cfg_if! {
                    if #[cfg(feature = "lossy_webp")] {
                        if webp_param.lossless {
                            src_image.write_with_encoder(
                                image::codecs::webp::WebPEncoder::new_lossless(cursor)
                            ).unwrap();
                        } else {
                            let mut webp_config = webp::WebPConfig::new().unwrap();
                            webp_config.lossless = c_int::from(webp_param.lossless);
                            webp_config.quality = webp_param.quality;

                            let encoder = webp::Encoder::from_image(&src_image).unwrap();
                            let result = encoder.encode_advanced(&webp_config);
                            if result.is_err() {
                                return Err(());
                            }
                            let result = result.unwrap();

                            let result = cursor.write_all(result.as_bytes());
                            if result.is_err() {
                                return Err(());
                            }
                        }
                    } else {
                        src_image.write_with_encoder(
                            image::codecs::webp::WebPEncoder::new_lossless(cursor)
                        ).unwrap();
                    }
                }
            }
            ImageBinaryFormat::Png => {
                result.content_extension = "png".to_string();
                result.content_type = "image/png".to_string();
                src_image.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
            }
            ImageBinaryFormat::Jpeg => {
                result.content_extension = "jpg".to_string();
                result.content_type = "image/jpeg".to_string();
                src_image.write_to(&mut cursor, image::ImageFormat::Jpeg).unwrap();
            }
        }

        Ok(result)
    });

    if result.is_err() {
        Err(())
    } else {
        Ok(result?)
    }
}
