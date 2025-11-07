use std::io::Cursor;
use image::{DynamicImage, ImageReader, ImageResult};
use crate::{FileStoreType, SourceParam};

pub fn read_img(src_param: SourceParam) -> ImageResult<DynamicImage> {
    match src_param.src_type {
        FileStoreType::File => {
            ImageReader::open(src_param.path.unwrap().clone())?
                .decode()
        },
        FileStoreType::Bytes => {
            ImageReader::new(Cursor::new(src_param.bytes.unwrap()))
                .with_guessed_format()?
                .decode()
        }
    }
}