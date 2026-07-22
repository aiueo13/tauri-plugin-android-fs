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
			    .header(http::header::CONTENT_TYPE, content_type.as_ref())
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
		content_type: std::borrow::Cow<'static, str>,
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
    request: http::Request<Vec<u8>>,
) -> std::result::Result<ProtocolResponse, ProtocolError> {

    let Some(config): Option<ProtocolConfigState> = app.try_state() else {
        return Err(ProtocolError::InternalServerError { 
            msg: "Missing protocol config state".into()
        })
    };

    let config = &config.thumbnail;

    if !config.enable {
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
        if !config.scope.as_ref().is_some_and(|s| s.is_allowed(path)) {
            return Err(ProtocolError::Forbidden)
        }
    }

    let is_head_method = match request.method() {
        &http::Method::GET => false,
        &http::Method::HEAD => true,
        _ => return Err(ProtocolError::MethodNotAllowed { 
            allow: resolve_allow_header([http::Method::GET, http::Method::HEAD]) 
        })
    };

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
        .map(|n| u32::min(n, 2048));
        
    let height = query
        .get("h")
        .and_then(|s| s.parse().ok())
        .map(|n| f64::ceil(n))
        .and_then(|n| f64_to_u32_for_size(n))
        .map(|n| u32::min(n, 2048));

    let (width, height) = match (width, height) {
        (Some(width), Some(height)) => (width, height),
        (Some(width), None) => (width, width),
        (None, Some(height)) => (height, height),
        (None, None) => (512, 512)
    };

    let format = query
        .get("f")
        .and_then(|s| ImageFormat::from_name(&s))
        .unwrap_or(ImageFormat::Webp);

    let Some(thumbnail) = app
        .android_fs_async()
        .impls()
        .get_file_thumbnail(&uri, Size { width, height }, format).await
        .map_err(|_| ProtocolError::NotFound)? else {

        return Err(ProtocolError::NotFound)
    };

    Ok(ProtocolResponse::Ok { 
        content_type: format.mime_type().into(), 
        content_len: thumbnail.len() as u64, 
        body: match is_head_method {
            true => Vec::new(),
            false => thumbnail
        }, 
    })
}

fn f64_to_u32_for_size(v: f64) -> Option<u32> {
    if v.is_finite() && 0.0 <= v && v <= u32::MAX as f64 {
        Some(v as u32)
    } 
    else {
        None
    }
}