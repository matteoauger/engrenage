use std::path::Path;

use axum::{
    http::{Uri, header},
    response::{IntoResponse, Response},
};
use rust_embed::{EmbeddedFile, RustEmbed};

use crate::error::AppError;

/// Static SPA assets, embedded into the binary at build time.
/// In debug builds rust-embed reads the folder from disk instead.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/dist"]
#[allow_missing = true]
struct Assets;

/// Whether the SPA bundle was built (an `index.html` exists). Public so
/// integration tests can branch on it, mirroring the runtime warning.
pub fn has_index() -> bool {
    Assets::get("index.html").is_some()
}

fn serve(file: EmbeddedFile) -> Response {
    let content_type = file.metadata.mimetype().to_owned();
    ([(header::CONTENT_TYPE, content_type)], file.data.to_vec()).into_response()
}

pub async fn handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = Assets::get(path) {
        return serve(file);
    }

    // A missing path with an extension looks like an asset request (script,
    // stylesheet, image, ...): no SPA fallback makes sense there, fail clean
    // instead of serving HTML that would surface as a JS parse error.
    if Path::new(path).extension().is_some() {
        return AppError::NotFound(format!("no such asset: {path}")).into_response();
    }

    // Extension-less path: SPA navigation — hand the URL to the client
    // router by serving index.html.
    match Assets::get("index.html") {
        Some(file) => serve(file),
        None => AppError::NotFound("SPA assets not built — run `npm run build -w web`".to_owned())
            .into_response(),
    }
}
