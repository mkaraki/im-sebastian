use image::{DynamicImage, GenericImageView};
use crate::{ResizeFilterType, ResizeMode, ResizeParam};
use cfg_if::cfg_if;

#[cfg(feature = "fast_image_resize")]
fn resize_with_fast_image_resize(src_image: &DynamicImage, resize_param: &ResizeParam, new_width: u32, new_height: u32) -> Result<DynamicImage, ()> {
    use fast_image_resize::images::Image;
    use fast_image_resize::{IntoImageView, Resizer, ResizeOptions};
    use crate::raw_image_decoder;

    let mut dst_image = Image::new(
        new_width,
        new_height,
        src_image.pixel_type().unwrap(),
    );

    let mut resizer = Resizer::new();
    cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            cfg_if! {
                if #[cfg(target_feature = "avx2")] {
                    unsafe {
                        resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Avx2);
                    }
                } else if #[cfg(target_feature = "sse4.1")] {
                    unsafe {
                        resizer.set_cpu_extensions(fast_image_resize::CpuExtensions::Sse4_1);
                    }
                }
            }
        }
    }

    let opt = ResizeOptions {
        algorithm: ResizeFilterType::as_fast_image_resize_resize_alg(&resize_param.resize_filter),
        cropping: Default::default(),
        mul_div_alpha: false,
    };

    resizer.resize(src_image, &mut dst_image, &Some(opt)).unwrap();

    let im_decoder_proxy = raw_image_decoder::RawImageDecoderProxy::new(
        dst_image.into_vec(),
        src_image.color(),
        (new_width, new_height),
    );

    let res = DynamicImage::from_decoder(im_decoder_proxy);
    if res.is_err() {
        return Err(());
    }

    Ok(res.unwrap())
}

#[cfg(not(feature = "fast_image_resize"))]
fn resize_with_image_crate(src_image: &DynamicImage, resize_param: &ResizeParam, new_width: u32, new_height: u32) -> DynamicImage {
    src_image.resize(new_width, new_height, ResizeFilterType::as_image_filter_type(&resize_param.resize_filter))
}

pub fn resize(src_image: &DynamicImage, resize_param: &ResizeParam) -> Result<DynamicImage, ()> {
    let src_dim = src_image.dimensions();
    let (new_width, new_height) = calc_new_size(src_dim, resize_param);

    if src_dim.0 == new_width && src_dim.1 == new_height  {
        return Ok(src_image.clone());
    }

    match resize_param.resize_mode {
        ResizeMode::Do | ResizeMode::ContainAndKeepAspectRatioIfLarger => {
            cfg_if! {
                if #[cfg(feature = "fast_image_resize")] {
                    resize_with_fast_image_resize(src_image, resize_param, new_width, new_height)
                } else {
                    Ok(resize_with_image_crate(src_image, resize_param, new_width, new_height))
                }
            }
        }
    }
}

fn calc_new_size(src_dimensions: (u32, u32), resize_param: &ResizeParam) -> (u32, u32) {
    let src_width: u32 = src_dimensions.0;
    let src_height: u32 = src_dimensions.1;

    let new_width: u32;
    let new_height: u32;

    match resize_param.resize_mode {
        ResizeMode::Do => {
            new_width  = resize_param.resize_width;
            new_height = resize_param.resize_height;
        }
        ResizeMode::ContainAndKeepAspectRatioIfLarger => {
            (new_width, new_height) = calc_contain_size(
                src_width, src_height, resize_param.resize_width, resize_param.resize_height
            );
        }
    }

    (new_width, new_height)
}

fn calc_contain_size(src_width: u32, src_height: u32, dest_width: u32, dest_height: u32) -> (u32, u32) {
    let dest_width = dest_width as f32;
    let dest_height = dest_height as f32;

    let mut new_width = src_width as f32;
    let mut new_height = src_height as f32;

    if new_width > dest_width {
        let new_ratio = dest_width / new_width;
        new_height *= new_ratio;
        new_width = dest_width;
    }

    if new_height > dest_height {
        let new_ratio = dest_height / new_height;
        new_width *= new_ratio;
        new_height = dest_height;
    }

    (new_width.ceil() as u32, new_height.ceil() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contain_test_a4portrait_to_square() {
        let res = calc_contain_size(2894, 4093, 128, 128);
        assert_eq!(res, (91, 128));
    }

    #[test]
    fn contain_test_a4portrait_to_1by2portrait() {
        let res = calc_contain_size(2894, 4093, 128, 256);
        assert_eq!(res, (128, 182));
    }
}
