use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use wry::http::{Request, Response};

const MERMAID_JS: &[u8] = include_bytes!("../assets/mermaid.min.js");

pub fn handle_asset_request(request: Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();

    if uri.contains("__mermaid.min.js") {
        return Response::builder()
            .status(200)
            .header("Content-Type", "application/javascript; charset=utf-8")
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache")
            .body(Cow::Borrowed(MERMAID_JS))
            .unwrap_or_else(|_| not_found());
    }
    
    // Extract path from URI (e.g. "mdasset://localhost/E:/path/to/img.png" or "http://mdasset.localhost/E:/path/to/img.png")
    let raw_path = if let Some(idx) = uri.find("://") {
        let after_scheme = &uri[idx + 3..];
        if let Some(slash_idx) = after_scheme.find('/') {
            &after_scheme[slash_idx..]
        } else {
            ""
        }
    } else {
        request.uri().path()
    };

    // Strip query string and fragment
    let no_query = raw_path.split('?').next().unwrap_or(raw_path).split('#').next().unwrap_or(raw_path);

    let decoded_path = percent_encoding::percent_decode_str(no_query)
        .decode_utf8_lossy()
        .to_string();

    let mut clean_path = decoded_path.as_str();
    // On Windows, "/C:/..." or "/E:/..." needs leading slash removed
    if clean_path.starts_with('/') && clean_path.len() > 2 && clean_path.chars().nth(2) == Some(':') {
        clean_path = &clean_path[1..];
    }

    let file_path = PathBuf::from(clean_path);

    if file_path.is_file() {
        if let Ok(bytes) = fs::read(&file_path) {
            let mime = get_mime_type(&file_path);
            return Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
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

fn get_mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        _ => "application/octet-stream",
    }
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
        let svg_path = Path::new("images/vector.svg");
        if svg_path.exists() {
            let canonical = svg_path.canonicalize().unwrap();
            let path_str = canonical.to_string_lossy().replace('\\', "/");
            let clean = path_str.trim_start_matches("//?/").trim_start_matches(r"\\?\");
            let uri = format!("http://mdasset.localhost/{}", clean);
            let req = Request::builder().uri(uri).body(Vec::new()).unwrap();
            let res = handle_asset_request(req);
            assert_eq!(res.status(), 200);
            assert_eq!(res.headers().get("Content-Type").unwrap(), "image/svg+xml");
        }
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
}

