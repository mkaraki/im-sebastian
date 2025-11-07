pub enum FileStoreType {
    File,
    Bytes,
}

pub struct SourceParam {
    pub src_type: FileStoreType,
    pub path: Option<String>,
    pub bytes: Option<Vec<u8>>,
}

pub enum ImageBinaryFormat {
    Webp,
    Png,
    Jpeg,
}

pub struct WebpExportParam {
    #[cfg(feature = "lossy_webp")]
    pub lossless: bool,
    #[cfg(feature = "lossy_webp")]
    pub quality: f32,
}

pub enum ResizeMode {
    Do,
    ContainAndKeepAspectRatioIfLarger,
}

pub struct ConvertParam {
    pub export_format: ImageBinaryFormat,
    pub webp_export_param: Option<WebpExportParam>,
    pub resize_param: Option<ResizeParam>,
}

pub struct ResizeParam {
    pub resize_mode: ResizeMode,
    pub resize_width: u32,
    pub resize_height: u32,
    pub resize_filter: ResizeFilterType,
}

pub enum ResizeFilterType {
    Nearest,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

impl ResizeFilterType {
    pub fn as_image_filter_type(&self) -> image::imageops::FilterType {
        use ResizeFilterType::*;
        use image::imageops::FilterType;
        match self {
            Nearest => FilterType::Nearest,
            CatmullRom => FilterType::CatmullRom,
            Gaussian => FilterType::Gaussian,
            Lanczos3 => FilterType::Lanczos3,
        }
    }

    pub fn as_fast_image_resize_resize_alg(&self) -> fast_image_resize::ResizeAlg {
        use ResizeFilterType::*;
        use fast_image_resize::{ResizeAlg, FilterType};
        match self {
            Nearest => ResizeAlg::Nearest,
            CatmullRom => ResizeAlg::Interpolation(FilterType::CatmullRom),
            Gaussian => ResizeAlg::Interpolation(FilterType::Gaussian),
            Lanczos3 => ResizeAlg::Interpolation(FilterType::Lanczos3),
        }
    }
}


#[derive(Default)]
pub struct ServedImage {
    pub content_type: String,
    pub content_extension: String,
    pub content: Vec<u8>,
}
