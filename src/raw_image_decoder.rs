use std::io::{Write};
use image::{ColorType, EncodableLayout, ImageDecoder, ImageError, ImageResult};

pub struct RawImageDecoderProxy {
    raw_image: Vec<u8>,
    color_type: ColorType,
    size: (u32, u32),
}

impl RawImageDecoderProxy {
    #[allow(dead_code)]
    pub fn new(raw_image: Vec<u8>, color_type: ColorType, size: (u32, u32)) -> Self {
        RawImageDecoderProxy {
            raw_image,
            color_type,
            size,
        }
    }
}

impl ImageDecoder for RawImageDecoderProxy {
    fn dimensions(&self) -> (u32, u32) {
        self.size
    }

    fn color_type(&self) -> ColorType {
        self.color_type
    }

    fn read_image(self, mut buf: &mut [u8]) -> ImageResult<()>
    where
        Self: Sized
    {
        let res = buf.write_all(self.raw_image.as_bytes());
        if res.is_err() {
            return Err(ImageError::IoError(res.err().unwrap()));
        }

        Ok(())
    }

    fn read_image_boxed(self: Box<Self>, mut buf: &mut [u8]) -> ImageResult<()> {
        let res = buf.write_all(self.raw_image.as_bytes());
        if res.is_err() {
            return Err(ImageError::IoError(res.err().unwrap()));
        }

        Ok(())
    }
}
