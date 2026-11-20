use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use uuid::Uuid;
use wry::http::{Request, Response};

const MERMAID_JS: &[u8] = include_bytes!("../assets/mermaid.min.js");
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/app_icon_48.png");
static REGISTERED_ASSETS: OnceLock<Mutex<HashMap<String, PathBuf>>> = OnceLock::new();

pub fn register_asset(path: PathBuf) -> Option<String> {
    if !is_supported_image(&path) {
        return None;
    }

    let token = format!("asset-{}", Uuid::new_v4().simple());
    REGISTERED_ASSETS
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .expect("asset registry mutex poisoned")
        .insert(token.clone(), path);
    Some(token)
}

pub fn handle_asset_request(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();

    if uri.contains("__mermaid.min.js") {
        return Response::builder()
            .status(200)
            .header("Content-Type", "application/javascript; charset=utf-8")
            .header("Cache-Control", "no-cache")
            .body(Cow::Borrowed(MERMAID_JS))
            .unwrap_or_else(|_| not_found());
    }

    if uri.contains("__app_icon.png") {
        return Response::builder()
            .status(200)
            .header("Content-Type", "image/png")
            .header("Cache-Control", "no-cache")
            .body(Cow::Borrowed(APP_ICON_PNG))
            .unwrap_or_else(|_| not_found());
    }
    
    let token = request.uri().path().trim_start_matches('/');
    let Some(file_path) = REGISTERED_ASSETS
        .get()
        .and_then(|assets| assets.lock().ok())
        .and_then(|assets| assets.get(token).cloned())
    else {
        return not_found();
    };

    if file_path.is_file() {
        if let Ok(bytes) = fs::read(&file_path) {
            let mime = get_mime_type(&file_path);
            return Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .header("Content-Security-Policy", "sandbox; default-src 'none'")
                .header("X-Content-Type-Options", "nosniff")
                .header("Cache-Control", "no-cache")
                .body(Cow::Owned(bytes))
                .unwrap_or_else(|_| not_found());
        }
    }

    not_found()
}

fn not_found() -> Response<Cow<'static, [u8]>> {
    Response::builder()
        .status(404)
        .header("Content-Type", "text/plain")
        .body(Cow::Borrowed(b"Asset not found" as &[u8]))
        .unwrap()
}

const DEFAULT_MIME_TYPE: &str = "application/octet-stream";
const MIME_TYPES: &[(&str, &str)] = &[
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("svg", "image/svg+xml"),
    ("webp", "image/webp"),
    ("avif", "image/avif"),
    ("bmp", "image/bmp"),
    ("ico", "image/x-icon"),
    ("css", "text/css; charset=utf-8"),
    ("js", "application/javascript; charset=utf-8"),
];

fn get_mime_type(path: &Path) -> &'static str {
    let Some(extension) = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
    else {
        return DEFAULT_MIME_TYPE;
    };

    MIME_TYPES
        .iter()
        .find(|(candidate, _)| *candidate == extension)
        .map_or(DEFAULT_MIME_TYPE, |(_, mime_type)| *mime_type)
}

fn is_supported_image(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .is_some_and(|extension| {
            MIME_TYPES
                .iter()
                .any(|(candidate, mime)| *candidate == extension && mime.starts_with("image/"))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mime_types() {
        assert_eq!(get_mime_type(Path::new("test.png")), "image/png");
        assert_eq!(get_mime_type(Path::new("test.jpg")), "image/jpeg");
        assert_eq!(get_mime_type(Path::new("test.jpeg")), "image/jpeg");
        assert_eq!(get_mime_type(Path::new("test.svg")), "image/svg+xml");
        assert_eq!(get_mime_type(Path::new("test.webp")), "image/webp");
        assert_eq!(get_mime_type(Path::new("test.gif")), "image/gif");
        assert_eq!(get_mime_type(Path::new("test.avif")), "image/avif");
        assert_eq!(get_mime_type(Path::new("test.bmp")), "image/bmp");
        assert_eq!(get_mime_type(Path::new("test.ico")), "image/x-icon");
    }

    #[test]
    fn test_handle_asset_svg() {
        let svg_path = PathBuf::from("images/vector.svg");
        let token = register_asset(svg_path.clone()).unwrap();
        let uri = format!("http://mdasset.localhost/{token}");
        let req = Request::builder().uri(uri).body(Vec::new()).unwrap();
        let res = handle_asset_request(req);
        assert_eq!(res.status(), if svg_path.exists() { 200 } else { 404 });
        if svg_path.exists() {
            assert_eq!(res.headers().get("Content-Type").unwrap(), "image/svg+xml");
        }
    }

    #[test]
    fn register_asset_rejects_non_image_paths() {
        assert!(register_asset(PathBuf::from("C:/secret.txt")).is_none());
        assert!(register_asset(PathBuf::from("C:/script.js")).is_none());
    }

    #[test]
    fn unregistered_asset_paths_are_rejected() {
        let uri = "http://mdasset.localhost/C:/Windows/win.ini";
        let req = Request::builder().uri(uri).body(Vec::new()).unwrap();
        let res = handle_asset_request(req);
        assert_eq!(res.status(), 404);
        assert!(res.headers().get("Access-Control-Allow-Origin").is_none());
    }

    #[test]
    fn test_handle_asset_mermaid() {
        let uri = "http://mdasset.localhost/__mermaid.min.js";
        let req = Request::builder().uri(uri).body(Vec::new()).unwrap();
        let res = handle_asset_request(req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), "application/javascript; charset=utf-8");
        assert!(!res.body().is_empty());
    }

    #[test]
    fn test_handle_asset_app_icon() {
        let uri = "http://mdasset.localhost/__app_icon.png";
        let req = Request::builder().uri(uri).body(Vec::new()).unwrap();
        let res = handle_asset_request(req);
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers().get("Content-Type").unwrap(), "image/png");
        assert!(!res.body().is_empty());
    }
}

