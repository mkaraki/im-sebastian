# im-sebastian

Simple image read/convert library.

## Sample Usage

Simple image parse, and resize code:

```rust
use im_sebastian::{FileStoreType, ResizeFilterType, ResizeParam};
fn main() {
    let src_param = im_sebastian::SourceParam {
        src_type: FileStoreType::Bytes,
        path: None,
        bytes: Some(source_image_bytes),
    };
    let convert_param = im_sebastian::ConvertParam {
        export_format: im_sebastian::ImageBinaryFormat::Png,
        webp_export_param: None,
        resize_param: Some(ResizeParam {
            resize_mode: im_sebastian::ResizeMode::ContainAndKeepAspectRatioIfLarger,
            resize_width: 500,
            resize_height: 500,
            resize_filter: ResizeFilterType::Nearest,
        })
    };
    let processed_image_data = im_sebastian::read_and_convert(src_param, convert_param).unwrap();
    let processed_image_bytes: Vec<u8> = processed_image_data.content;
}
```

Other samples are in [`img-server` sample webserver code](img-server).
