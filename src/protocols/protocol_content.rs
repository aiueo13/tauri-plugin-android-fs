use crate::*;
use super::*;
use tauri::{http, Manager as _};


pub const URI_SCHEME: &'static str = "android-fs-content";

pub fn protocol<R: tauri::Runtime>(
    ctx: tauri::UriSchemeContext<'_, R>,
    request: tauri::http::Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {

    let app = ctx.app_handle().clone();

    tauri::async_runtime::spawn(async move {
        responder.respond(match create_response(app, request).await {
            Ok(ProtocolResponse::Ok { body, content_type, content_len }) => http::Response::builder()
                .status(http::StatusCode::OK)
                .header(http::header::ACCEPT_RANGES, "bytes")
			    .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::CONTENT_LENGTH, content_len)
                .body(body)
                .unwrap_or_default(),

            Ok(ProtocolResponse::Part { body, content_type, content_len, content_range }) => http::Response::builder()
                .status(http::StatusCode::PARTIAL_CONTENT)
			    .header(http::header::ACCEPT_RANGES, "bytes")
                .header(http::header::CONTENT_RANGE, content_range)
			    .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::CONTENT_LENGTH, content_len)
                .body(body)
                .unwrap_or_default(),

            Ok(ProtocolResponse::Multipart { body, content_type, content_len }) => http::Response::builder()
                .status(http::StatusCode::PARTIAL_CONTENT)
			    .header(http::header::ACCEPT_RANGES, "bytes")
			    .header(http::header::CONTENT_TYPE, content_type)
                .header(http::header::CONTENT_LENGTH, content_len)
                .body(body)
                .unwrap_or_default(),

		    Err(ProtocolError::NotSatisfiable { content_range }) => http::Response::builder()
                .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
			    .header(http::header::ACCEPT_RANGES, "bytes")
                .header(http::header::CONTENT_RANGE, content_range)
			    .header(http::header::CONTENT_LENGTH, 0)
                .body(Vec::new())
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
	Part {
		body: Vec<u8>,
        content_range: String,
		content_type: String,
        content_len: u64,
	},
    Multipart {
		body: Vec<u8>,
		content_type: String,
        content_len: u64,
	},
}

enum ProtocolError {
    NotSatisfiable {
	    content_range: String,
	},
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
            msg: "Missing protocol-content feature".into()
        })
    };

    if !config.enable_content {
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
        if !config.content_scope.as_ref().is_some_and(|s| s.is_allowed(path)) {
            return Err(ProtocolError::Forbidden)
        }
    }

    let (mut file, mime_type, len) = resolve_content(uri, app).await?;

    let method = request.method().clone();
    let ranges = request
        .headers()
        .get(http::header::RANGE)
        .map(|v| resolve_range(v.as_bytes(), len))
        .transpose()?
        .unwrap_or_else(Vec::new);
        
    tauri::async_runtime::spawn_blocking(move || {
        match ranges.len() {
            0 => create_entire_response(&mut file, len, mime_type, method),
            1 => create_part_response(&mut file, len, mime_type, method, ranges[0]),
            _ => create_multipart_response(&mut file, len, mime_type, method, ranges),
        }
    })
    .await
    .map_err(|_| ProtocolError::InternalServerError { msg: "Failed to execute blocking task".into() })?
}

// TODO: ファイルやメタデータをキャッシュする
async fn resolve_content<R: tauri::Runtime>(
    uri: FileUri,
    app: tauri::AppHandle<R>,
) -> std::result::Result<(std::fs::File, String, u64), ProtocolError> {

    let api = app.android_fs_async();

    let Ok((mut file, mime_type, len)) = api
        .impls()
        .get_file_resource_for_content_protocol(&uri).await else {

        return Err(ProtocolError::NotFound)
    };

    let (file, len) = match len.and_then(std::num::NonZeroU64::new) {
        Some(len) => (file, len.get()),
        None => tauri::async_runtime::spawn_blocking(move || {
                use std::io::{Seek as _, SeekFrom};
                let len = file.seek(SeekFrom::End(0)).ok()?;
                Some((file, len as u64))
            })
            .await
            .ok()
            .and_then(|r| r)
            .ok_or_else(|| ProtocolError::InternalServerError {
                msg: "Failed to seek content".into()
            })?
    };

    let mime_type = mime_type.unwrap_or("application/octet-stream".to_string());

    Ok((file, mime_type, len))
}


fn create_entire_response<C: std::io::Read + std::io::Seek>(
    content: &mut C,
    content_len: u64,
    content_type: String,
    method: http::Method,
) -> std::result::Result<ProtocolResponse, ProtocolError> {

    let body = match method {
        http::Method::HEAD => Vec::new(),
        http::Method::GET => read_content(content, 0, content_len)?,
        _ => return Err(ProtocolError::MethodNotAllowed { 
            allow: resolve_allow_header([http::Method::GET, http::Method::HEAD]) 
        })
    };

    Ok(ProtocolResponse::Ok { body, content_type, content_len })
}

fn create_part_response<C: std::io::Read + std::io::Seek>(
    content: &mut C,
    content_len: u64,
    content_type: String,
    method: http::Method,
    range: HttpRange,
) -> std::result::Result<ProtocolResponse, ProtocolError> {

    let (range_start, range_end, range_len) = (range.start, range.end, range.len);
    let range_data = match method {
        http::Method::HEAD => Vec::new(),
        http::Method::GET => read_content(content, range_start, range_len)?,
        _ => return Err(ProtocolError::MethodNotAllowed { 
            allow: resolve_allow_header([http::Method::GET, http::Method::HEAD]) 
        })
    };

    Ok(ProtocolResponse::Part { 
        body: range_data, 
        content_type,
        content_len: range_len,
        content_range: format!("bytes {range_start}-{range_end}/{content_len}"),
    })
}

fn create_multipart_response<C: std::io::Read + std::io::Seek>(
    content: &mut C,
    content_len: u64,
    content_type: String,
    method: http::Method,
    ranges: impl IntoIterator<Item = HttpRange>
) -> std::result::Result<ProtocolResponse, ProtocolError> {

    let boundary = generate_multipart_boundary()?;
    let (body, content_len) = match method {
        http::Method::GET => {
            let body = create_multipart_body(content, content_len, &content_type, &boundary, ranges)?;
            let len = body.len() as u64;
            (body, len)
        },
        http::Method::HEAD => {
            let len = calc_multipart_body_len(content_len, &content_type, &boundary, ranges)?;
            (Vec::new(), len)
        },
        _ => return Err(ProtocolError::MethodNotAllowed { 
            allow: resolve_allow_header([http::Method::GET, http::Method::HEAD]) 
        })
    };

    Ok(ProtocolResponse::Multipart {
        content_type: format!("multipart/byteranges; boundary={boundary}"),
        content_len,
        body,
    })
}

fn create_multipart_body<C: std::io::Read + std::io::Seek>(
    content: &mut C,
    content_len: u64,
    content_type: &str,
    boundary: &str,
    ranges: impl IntoIterator<Item = HttpRange>
) -> std::result::Result<Vec<u8>, ProtocolError> {

    let mut buf = Vec::new();
    for range in ranges {
        let (range_start, range_end, range_len) = (range.start, range.end, range.len);
        let range_data = read_content(content, range_start, range_len)?;

        buf.extend_from_slice("--".as_bytes());
        buf.extend_from_slice(boundary.as_bytes());
        buf.extend_from_slice("\r\n".as_bytes());

        buf.extend_from_slice("Content-Type: ".as_bytes());
        buf.extend_from_slice(content_type.as_bytes());
        buf.extend_from_slice("\r\n".as_bytes());

        buf.extend_from_slice("Content-Range: bytes ".as_bytes());
        buf.extend_from_slice(format!("{range_start}-{range_end}/{content_len}").as_bytes());
        buf.extend_from_slice("\r\n".as_bytes());

        buf.extend_from_slice("\r\n".as_bytes());

        buf.extend_from_slice(&range_data);
        buf.extend_from_slice("\r\n".as_bytes());
    }
    buf.extend_from_slice("--".as_bytes());
    buf.extend_from_slice(boundary.as_bytes());
    buf.extend_from_slice("--".as_bytes());
    buf.extend_from_slice("\r\n".as_bytes());

    Ok(buf)
}

fn calc_multipart_body_len(
    content_len: u64,
    content_type: &str,
    boundary: &str,
    ranges: impl IntoIterator<Item = HttpRange>
) -> std::result::Result<u64, ProtocolError> {

    let mut total_len = 0;
    for range in ranges {
        let (range_start, range_end, range_len) = (range.start, range.end, range.len);

        total_len += "--".len() as u64;
        total_len += boundary.len() as u64;
        total_len += "\r\n".len() as u64;

        total_len += "Content-Type: ".len() as u64;
        total_len += content_type.len() as u64;
        total_len += "\r\n".len() as u64;

        total_len += "Content-Range: bytes ".len() as u64;
        total_len += format!("{range_start}-{range_end}/{content_len}").len() as u64;
        total_len += "\r\n".len() as u64;

        total_len += "\r\n".len() as u64;

        total_len += range_len;
        total_len += "\r\n".len() as u64;
    }
    total_len += "--".len() as u64;
    total_len += boundary.len() as u64;
    total_len += "--".len() as u64;
    total_len += "\r\n".len() as u64;

    Ok(total_len)
}


fn read_content<C: std::io::Seek + std::io::Read>(
    content: &mut C,
    offset: u64,
    len: u64,
) -> std::result::Result<Vec<u8>, ProtocolError> {

    let mut inner = || {
        use std::io::Read as _;
        let len_usize = usize::try_from(len).ok()?;
        let mut buf = Vec::with_capacity(len_usize);
        content.seek(std::io::SeekFrom::Start(offset)).ok()?;
        content.take(len).read_to_end(&mut buf).ok()?;
        Some(buf)
    };

    match inner() {
        Some(r) => Ok(r),
        None => Err(ProtocolError::InternalServerError { 
            msg: "Failed to read content data".into()
        })
    }
}

fn generate_multipart_boundary() -> std::result::Result<String, ProtocolError> {
    fn inner() -> Option<String> {
        let mut x = [0_u8; 30];
        getrandom::fill(&mut x).ok()?;

        let mut buf = String::with_capacity(30 * 2);

        for byte in x {
            use std::fmt::Write;
            write!(&mut buf, "{:02x}", byte).ok()?;
        }
        Some(buf)
    }

    match inner() {
        Some(buf) => Ok(buf),
        None => Err(ProtocolError::InternalServerError {
            msg: "Failed to generate multipart boundary".into()
        })
    }
}

const RANGE_MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Clone, Copy)]
struct HttpRange {
    start: u64,
    end: u64,
    len: u64,
}

fn resolve_range(header: &[u8], entire_len: u64) -> std::result::Result<Vec<HttpRange>, ProtocolError> {
    http_range::HttpRange::parse_bytes(header, entire_len)
        .map_err(|_| ProtocolError::NotSatisfiable { content_range: format!("bytes */{entire_len}") })
        .map(|r| r.into_iter().map(|r| {
            let start = r.start;
            let len = u64::min(r.length, RANGE_MAX_BYTES);
            let end = (start + len).saturating_sub(1);
            HttpRange { start, end, len }
        }).collect())
}