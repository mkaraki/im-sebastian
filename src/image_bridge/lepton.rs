use std::ffi::OsString;
use std::io::{BufRead, Cursor, Seek};
use image::codecs::jpeg::JpegDecoder;
use image::error::{DecodingError, ImageFormatHint};
use image::{ColorType, ImageDecoder, ImageError, ImageResult};

pub struct LeptonDecoder {
    jpeg_decoder: JpegDecoder<Cursor<Vec<u8>>>,
}

impl LeptonDecoder {
    pub fn new<R: BufRead + Seek>(mut r: R) -> ImageResult<LeptonDecoder> {
        let mut lepton_data = Vec::new();
        r.read_to_end(&mut lepton_data)
            .map_err(ImageError::IoError)?;

        let enabled_features = lepton_jpeg::EnabledFeatures {
            progressive: true,
            reject_dqts_with_zeros: false,
            max_jpeg_width: 10000,
            max_jpeg_height: 10000,
            use_16bit_dc_estimate: false,
            use_16bit_adv_predict: false,
            accept_invalid_dht: true,
            max_threads: 2,
            max_jpeg_file_size: 500_000_000u32,
            stop_reading_at_eoi: false,
        };

        let mut jpeg_buffer = Vec::new();
        let res = lepton_jpeg::decode_lepton(
            &mut lepton_data.as_slice(),
            &mut jpeg_buffer,
            &enabled_features,
            &lepton_jpeg::DEFAULT_THREAD_POOL,
        );

        if let Err(e) = res {
            return Err(ImageError::Decoding(DecodingError::new(
                ImageFormatHint::Name("Lepton".to_string()),
                e,
            )));
        }

        let jpeg_reader = Cursor::new(jpeg_buffer);
        let jpeg_decoder = JpegDecoder::new(jpeg_reader)?;

        Ok(LeptonDecoder { jpeg_decoder })
    }
}

impl ImageDecoder for LeptonDecoder {
    fn dimensions(&self) -> (u32, u32) {
        self.jpeg_decoder.dimensions()
    }

    fn color_type(&self) -> ColorType {
        self.jpeg_decoder.color_type()
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()> {
        self.jpeg_decoder.read_image(buf)
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        self.jpeg_decoder.read_image(buf)
    }
}

pub fn register() {
    let ret = image::hooks::register_decoding_hook(
        OsString::from("lep"),
        Box::new(|r| {
            LeptonDecoder::new(r).map(|d| Box::new(d) as Box<dyn ImageDecoder>)
        }),
    );
    if ret {
        image::hooks::register_format_detection_hook(OsString::from("lep"), &[0xcf, 0x84], None);
        // zlib lepton
        // image::hooks::register_format_detection_hook(OsString::from("lep"), &[0xce, 0xb6], None);
    }
}
