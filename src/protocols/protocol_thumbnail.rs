use crate::*;
use super::*;
use tauri::{http, Manager as _};


pub const URI_SCHEME: &'static str = "android-fs-thumbnail";

pub fn protocol<R: tauri::Runtime>(
    ctx: tauri::UriSchemeContext<'_, R>, 
    request: tauri::http::Request<Vec<u8>>, 
    responder: tauri::UriSchemeResponder
) {
    
    let app = ctx.app_handle().clone();

    tauri::async_runtime::spawn(async move {
        responder.respond(match create_response(app, request).await {
            Ok(ProtocolResponse::Ok { body, content_type, content_len }) => http::Response::builder()
                .status(http::StatusCode::OK)
			    .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::CONTENT_LENGTH, content_len)
                .body(body)
                .unwrap_or_default(),

            Err(ProtocolError::MethodNotAllowed { allow }) => http::Response::builder()
                .status(http::StatusCode::METHOD_NOT_ALLOWED)
                .header(http::header::ALLOW, allow)
                .header(http::header::CONTENT_LENGTH, 0)
                .body(Vec::new())
                .unwrap_or_default(),

            Err(ProtocolError::BadRequest { msg }) => http::Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
			    .header(http::header::CONTENT_LENGTH, msg.len())
                .body(msg.to_string().into_bytes())
                .unwrap_or_default(),

            Err(ProtocolError::InternalServerError { msg }) => http::Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
			    .header(http::header::CONTENT_LENGTH, msg.len())
                .body(msg.to_string().into_bytes())
                .unwrap_or_default(),

            Err(ProtocolError::Forbidden) => http::Response::builder()
                .status(http::StatusCode::FORBIDDEN)
                .header(http::header::CONTENT_LENGTH, 0)
                .body(Vec::new())
                .unwrap_or_default(),

            Err(ProtocolError::NotFound) => http::Response::builder()
                .status(http::StatusCode::NOT_FOUND)
                .header(http::header::CONTENT_LENGTH, 0)
                .body(Vec::new())
                .unwrap_or_default(),
        });
    });
}


enum ProtocolResponse {
	Ok {
		body: Vec<u8>,
		content_type: String,
        content_len: u64,
	},
}

enum ProtocolError {
    MethodNotAllowed {
	    allow: String,
	},
    BadRequest {
        msg: std::borrow::Cow<'static, str>,
    },
    InternalServerError {
        msg: std::borrow::Cow<'static, str>,
    },
    Forbidden,
    NotFound,
}

async fn create_response<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    request: http::Request<Vec<u8>>
) -> std::result::Result<ProtocolResponse, ProtocolError> {

    let Some(config): Option<ProtocolConfigState> = app.try_state() else {
        return Err(ProtocolError::InternalServerError { 
            msg: "Missing protocol-thumbnail feature".into()
        })
    };

    if !config.enable_thumbnail {
        return Err(ProtocolError::Forbidden)
    }

    let Some(uri) = percent_encoding::percent_decode_str(request.uri().path().trim_start_matches('/'))
        .decode_utf8().ok()
        .and_then(|s| serde_json::from_str::<AfsUriOrFsPath>(&s).ok())
        .and_then(|s| s.try_into_content_or_safe_file_scheme_uri().ok()) else {

        return Err(ProtocolError::BadRequest {
            msg: "Bad URI format".into()
        })
    };
    
    if let Some(path) = uri.to_path() {
        if !config.thumbnail_scope.as_ref().is_some_and(|s| s.is_allowed(path)) {
            return Err(ProtocolError::Forbidden)
        }
    }
    
    let method = request.method();
    let headers = request.headers();
    let query = request.uri()
        .query()
        .unwrap_or("")
        .split('&')
        .filter_map(|v| v.split_once('='))
        .collect::<std::collections::HashMap<&str, &str>>();

    let width = query
        .get("w")
        .and_then(|s| s.parse().ok())
        .map(|n| f64::ceil(n))
        .and_then(|n| f64_to_u32_for_size(n))
        .map(|n| u32::min(n, 1024));
        
    let height = query
        .get("h")
        .and_then(|s| s.parse().ok())
        .map(|n| f64::ceil(n))
        .and_then(|n| f64_to_u32_for_size(n))
        .map(|n| u32::min(n, 1024));

    let (width, height) = match (width, height) {
        (Some(width), Some(height)) => (width, height),
        (Some(width), None) => (width, width),
        (None, Some(height)) => (height, height),
        (None, None) => (256, 256)
    };

    let format = query
        .get("f")
        .and_then(|s| ImageFormat::from_name(&s))
        .unwrap_or_else(|| 
            headers
                .get(http::header::ACCEPT)
                .and_then(|accept| get_best_mime_type_from_accept_header(
                    accept.as_bytes(), 
                    &["image/jpeg", "image/jpg", "image/webp", "image/png"]
                ))
                .and_then(|m| ImageFormat::from_mime_type(m))
                .unwrap_or(ImageFormat::Jpeg)
        );
        
    let api = app.android_fs_async();
    let thumbnail = api
        .get_thumbnail(&uri, Size { width, height }, format).await
        .map_err(|_| ProtocolError::NotFound)?
        .ok_or_else(|| ProtocolError::NotFound)?;
    
    match method {
        &http::Method::GET => Ok(ProtocolResponse::Ok { 
            content_type: format.mime_type().into(), 
            content_len: thumbnail.len() as u64,
            body: thumbnail, 
        }),
        &http::Method::HEAD => Ok(ProtocolResponse::Ok { 
            content_type: format.mime_type().into(), 
            content_len: thumbnail.len() as u64, 
            body: Vec::new(), 
        }),
        _ => Err(ProtocolError::MethodNotAllowed { 
            allow: resolve_allow_header([http::Method::GET, http::Method::HEAD]) 
        })
    }
}

fn get_best_mime_type_from_accept_header<'a>(
    accept_header_value: &[u8],
    supported: &[&'a str],
) -> Option<&'a str> {

    let mut best: Option<&'a str> = None;
    let mut best_q = 0.0;

    for &s in supported {
        let mut current_q = 0.0;
        let mut highest_spec = 0;

        for item in accept_header_value.split(|&b| b == b',').take(64) {
            let Ok(item_str) = std::str::from_utf8(item) else {
                continue;
            };

            let mut parts = item_str.split(';').map(str::trim);
            let mime = match parts.next() {
                Some(v) => v,
                None => continue,
            };

            let spec = if mime.eq_ignore_ascii_case(s) {
                3
            }
            else if mime.ends_with("/*") {
                let prefix = &mime[..mime.len() - 1];
                if prefix.len() <= s.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix) {
                    2
                } 
                else {
                    0
                }
            } 
            else if mime == "*/*" {
                1
            } 
            else {
                0
            };

            if highest_spec < spec {
                let mut q = 1.0;
                for p in parts {
                    if let Some(v) = p.strip_prefix("q=") {
                        if let Ok(parsed) = v.parse::<f32>() {
                            if parsed.is_finite() && 0.0 <= parsed && parsed <= 1.0 {
                                q = parsed;
                            }
                        }
                    }
                }
                highest_spec = spec;
                current_q = q;
            }
        }

        if 0.0 < current_q && best_q < current_q {
            best_q = current_q;
            best = Some(s);
        }
    }

    best
}

fn f64_to_u32_for_size(v: f64) -> Option<u32> {
    if v.is_finite() && 0.0 <= v && v <= u32::MAX as f64 {
        Some(v as u32)
    } 
    else {
        None
    }
}