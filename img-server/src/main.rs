use std::path::PathBuf;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, Response, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use opentelemetry_sdk::trace::SdkTracerProvider;
use path_clean::PathClean;
use tokio::fs;
use im_sebastian::{FileStoreType, ResizeFilterType, ResizeParam};

#[derive(Clone)]
struct AppStateInfo {
    canonical_base: Arc<PathBuf>,
}

async fn file_reader(
    state: AppStateInfo,
    user_path: String,
) -> Result<Vec<u8>, Response<Body>> {
    let full_path = state.canonical_base.join(&user_path);
    let cleaned_path = full_path.clean();

    if !cleaned_path.starts_with(state.canonical_base.as_ref()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path").into_response());
    }

    let src_image = fs::read(&cleaned_path).await;
    if src_image.is_err() {
        return Err((StatusCode::NOT_FOUND, "File not found").into_response());
    }

    Ok(src_image.unwrap())
}

async fn file_handler(
    State(state): State<AppStateInfo>,
    Path(user_path): Path<String>,
) -> impl IntoResponse {
    let src_image = file_reader(state, user_path).await;
    if src_image.is_err() {
        return src_image.err().unwrap();
    }
    let src_image = src_image.ok().unwrap();

    let src_param = im_sebastian::SourceParam{
        src_type: FileStoreType::Bytes,
        path: None,
        bytes: Some(src_image),
    };
    let convert_param = im_sebastian::ConvertParam{
        export_format: im_sebastian::ImageBinaryFormat::Png,
        webp_export_param: None,
        resize_param: None,
    };
    let processed_image_data = im_sebastian::read_and_convert(src_param, convert_param);
    if processed_image_data.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error: Unable to read/convert image").into_response();
    }
    let processed_image_data = processed_image_data.unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, processed_image_data.content_type.parse().unwrap());
    (StatusCode::OK, headers, processed_image_data.content).into_response()
}

async fn thumb_file_handler(
    State(state): State<AppStateInfo>,
    Path(user_path): Path<String>,
) -> impl IntoResponse {
    let src_image = file_reader(state, user_path).await;
    if src_image.is_err() {
        return src_image.err().unwrap();
    }
    let src_image = src_image.ok().unwrap();

    let src_param = im_sebastian::SourceParam{
        src_type: FileStoreType::Bytes,
        path: None,
        bytes: Some(src_image),
    };
    let convert_param = im_sebastian::ConvertParam{
        export_format: im_sebastian::ImageBinaryFormat::Png,
        webp_export_param: None,
        resize_param: Some(ResizeParam {
            resize_mode: im_sebastian::ResizeMode::ContainAndKeepAspectRatioIfLarger,
            resize_width: 128,
            resize_height: 128,
            resize_filter: ResizeFilterType::Nearest,
        })
    };
    let processed_image_data = im_sebastian::read_and_convert(src_param, convert_param);
    if processed_image_data.is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error: Unable to read/convert image").into_response();
    }
    let processed_image_data = processed_image_data.unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, processed_image_data.content_type.parse().unwrap());
    (StatusCode::OK, headers, processed_image_data.content).into_response()
}

#[tokio::main]
async fn main() {
    // Open telemetry config
    fn init_trace() -> SdkTracerProvider {
        let exporter = opentelemetry_stdout::SpanExporter::default();
            SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build()
    }
    opentelemetry::global::set_tracer_provider(init_trace());

    // Web server config
    let canonical_base = std::fs::canonicalize("./images")
        .expect("Failed to find base directory");

    let app_state = AppStateInfo {
        canonical_base: Arc::new(canonical_base),
    };

    let app = Router::new()
        .route("/thumbs/*path", get(thumb_file_handler))
        .route("/images/*path", get(file_handler))
        .with_state(app_state); // Pass the state to the handler

    // Listener config
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Listen: http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}