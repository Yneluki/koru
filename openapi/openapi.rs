use std::fs;
use utoipa::OpenApi;

/// This is a simple executable that can be used to generate the OpenApi JSON document.
/// The file will be generated at `./openapi/koru_openapi.json`.
///
/// It can be executed with
/// ```
/// cargo run --bin gen-openapi
/// ```
///
fn main() {
    let doc = koru::api::ApiDoc::openapi()
        .to_pretty_json()
        .expect("OpenAPI generation to succeed");
    fs::write("./openapi/koru_openapi.json", doc).expect("File generation to succeed");
}
