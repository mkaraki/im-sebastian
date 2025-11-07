use std::io::Cursor;
use image::{DynamicImage, ImageReader, ImageResult};
use crate::{FileStoreType, SourceParam};

pub fn read_img(src_param: SourceParam) -> ImageResult<DynamicImage> {
    match src_param.src_type {
        FileStoreType::File(p) => {
            ImageReader::open(p.path.clone())?
                .decode()
        },
        FileStoreType::Bytes(p) => {
            ImageReader::new(Cursor::new(p.bytes))
                .with_guessed_format()?
                .decode()
        }
    }
}