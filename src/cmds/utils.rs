use tauri::Manager as _;
use crate::*;
use super::*;


#[cfg(target_os = "android")]
pub async fn resolve_mime_type<'a, R: tauri::Runtime>(
    mime_type: Option<&'a str>,
    path: impl AsRef<str>,
    app: &tauri::AppHandle<R>,
) -> Result<std::borrow::Cow<'a, str>> {

    if let Some(mime_type) = mime_type {
        return Ok(mime_type.into())
    }

    let path = path.as_ref();
    let file_name = path.rsplit_once('/')
        .map(|(_, file_name)| file_name)
        .unwrap_or(path);

    if let Some((_, ext)) = file_name.rsplit_once('.') {
        let api = app.android_fs_async();
        if let Some(mime_type) = api.get_mime_type_from_extension(ext).await? {
            return Ok(mime_type.into())
        }
    }
    
    Ok("application/octet-stream".into())
}

#[cfg(target_os = "android")]
pub async fn resolve_picker_initial_location<R: tauri::Runtime>(
    initial_location: PickerInitialLocation,
    app: &tauri::AppHandle<R>,
) -> Result<FileUri> {

    let api = app.android_fs_async();
    let map_volume_id = |id: Option<&str>| -> Result<Option<StorageVolumeId>> {
        match id {
            Some(v) => Ok(Some(convert_to_storage_volume_id(v)?)),
            None => Ok(None),
        }
    };

    match initial_location {
        PickerInitialLocation::Any { uri } => {
            Ok(uri)
        },
        PickerInitialLocation::VolumeTop { volume_id } => {
            api.resolve_root_initial_location(
                map_volume_id(volume_id.as_deref())?.as_ref()
            ).await
        },
        PickerInitialLocation::PublicDir { base_dir, relative_path, volume_id } => {
            api.public_storage().resolve_initial_location(
                map_volume_id(volume_id.as_deref())?.as_ref(), 
                base_dir, 
                relative_path.as_deref().unwrap_or(""), 
                true,
            ).await
        },
    }
}

#[cfg(target_os = "android")]
pub fn convert_to_thumbnail_preferred_size(w: f64, h: f64) -> Result<Size> {
    if !w.is_finite() || !h.is_finite() {
        return Err(Error::with("non-finite width or height"));
    }
    if w <= 0.0 || h <= 0.0 {
        return Err(Error::with(format!("non-positive width or height: ({w}, {h})")));
    }

    const MAX: u32 = 1000;

    let width = u32::clamp(w.round() as u32, 1, MAX);
    let height = u32::clamp(h.round() as u32, 1, MAX);

    Ok(Size { width, height })
}

#[cfg(target_os = "android")]
pub fn convert_to_image_format(format: &str) -> Result<ImageFormat> {
    match format.to_ascii_lowercase().as_str() {
        "jpeg" | "jpg" => Ok(ImageFormat::Jpeg),
        "webp" => Ok(ImageFormat::Webp),
        "png" => Ok(ImageFormat::Png),
        _ => Err(Error::with(format!("unexpected image format: {format}")))
    }
}

#[cfg(target_os = "android")]
pub fn convert_to_storage_volume_id(id: &str) -> Result<StorageVolumeId> {
    serde_json::from_str(id).map_err(Into::into)
}

#[cfg(target_os = "android")]
pub fn convert_from_storage_volume_id(id: &StorageVolumeId) -> Result<String> {
    serde_json::to_string(id).map_err(Into::into)
}

#[cfg(target_os = "android")]
pub fn convert_time_to_f64_millis(time: std::time::SystemTime) -> Result<f64> {
    let duration = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::ZERO);

    Ok(duration.as_millis() as f64)
}

#[cfg(target_os = "android")]
pub fn convert_bytes_to_base64(bytes: &[u8]) -> Result<String> {
    use base64::engine::Engine;
    use base64::engine::general_purpose::STANDARD;

    Ok(STANDARD.encode(bytes))
}

#[cfg(target_os = "android")]
pub fn convert_bytes_to_data_url(bytes: &[u8], mime_type: &str) -> Result<String> {
    use base64::engine::Engine;
    use base64::engine::general_purpose::STANDARD;

    let mut buffer = format!("data:{mime_type};base64,");
    buffer.reserve_exact((bytes.len() * 4 / 3) + 4);
    STANDARD.encode_string(bytes, &mut buffer);
    Ok(buffer)
}

#[cfg(target_os = "android")]
pub fn convert_base64_to_data_url(base64: &str, mime_type: &str) -> Result<String> {
    let mut buffer = format!("data:{mime_type};base64,");
    buffer.reserve_exact(base64.len());
    buffer.push_str(base64);
    Ok(buffer)
}


#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum WriteFileStreamEventInput {
    Open {
        uri: AfsUriOrFsPath,
        supports_raw_ipc_request_body: bool,
        options: WriteFileStreamEventInputOptions
    },
    Write {
        id: tauri::ResourceId,
        data: Vec<u8>,
    },
    Close {
        id: tauri::ResourceId,
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub struct WriteFileStreamEventInputOptions {
    pub create: bool,
}

#[cfg(target_os = "android")]
impl<'a> TryInto<WriteFileStreamEventInput> for tauri::ipc::Request<'a> {
    type Error = Error;

    fn try_into(self) -> Result<WriteFileStreamEventInput> {
        let get_header_value = |header_name: &str| -> Result<_> {
            self.headers()
                .get(header_name)
                .ok_or_else(|| Error::missing_value(header_name))
        };
        
        let event_type = get_header_value("eventType")?.to_str()?;

        match event_type {
            "Open" => {
                // 呼び出し時に body として与えられた判定用の payload をチェックして
                // 生の body を受け取り可能かどうかを調べる。
                // <https://github.com/tauri-apps/tauri/issues/10573>
                let supports_raw_ipc_request_body = match self.body() {
                    tauri::ipc::InvokeBody::Json(_) => false,
                    tauri::ipc::InvokeBody::Raw(_) => true,
                };

                let uri = get_header_value("uri")
                    .map(|s| percent_encoding::percent_decode(s.as_ref()))
                    .and_then(|s| s.decode_utf8().map_err(Into::into))
                    .and_then(|s| serde_json::from_str(&s).map_err(Into::into))?;
               
                let options = get_header_value("options")
                    .map(|s| percent_encoding::percent_decode(s.as_ref()))
                    .and_then(|s| s.decode_utf8().map_err(Into::into))
                    .and_then(|s| serde_json::from_str(&s).map_err(Into::into))?;

                Ok(WriteFileStreamEventInput::Open { uri, options, supports_raw_ipc_request_body })
            },
            "Write" => {
                let id = get_header_value("id")?
                    .to_str()?
                    .parse::<u32>()?;

                let data = match self.body() {
                    tauri::ipc::InvokeBody::Raw(body) => {
                        body.clone()
                    },
                    tauri::ipc::InvokeBody::Json(body) => {
                        let data = body
                            .get("data")
                            .ok_or_else(|| Error::missing_value("data"))?
                            .as_str()
                            .ok_or_else(|| Error::invalid_type("data"))?;

                        let b64 = match data.starts_with("data:") {
                            // data URL
                            true => {
                                let comma_i = data
                                    .find(",")
                                    .ok_or_else(|| Error::with("invalid Data URL"))?;

                                let (_, b64) = data.split_at(comma_i + 1);
                                b64
                            },
                            // base64
                            false => data,
                        };

                        // TODO: データが大きい場合は別スレッドに逃す
                        use base64::engine::Engine;
                        base64::engine::general_purpose::STANDARD.decode(b64)?
                    },
                };

                Ok(WriteFileStreamEventInput::Write { id, data })
            },
            "Close" => {
                let id = get_header_value("id")?
                    .to_str()?
                    .parse::<u32>()?;

                Ok(WriteFileStreamEventInput::Close { id })
            },
            value => Err(Error::invalid_value(value))
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum WriteFileStreamEventOutput {
    Open {
        id: tauri::ResourceId,

        #[serde(rename="supportsRawIpcRequestBody")]
        supports_raw_ipc_request_body: bool
    },
    Write(()),
    Close(()),
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum ReadFileStreamEventInput {
    Open {
        uri: AfsUriOrFsPath,
    },
    Read {
        id: tauri::ResourceId,
        len: u64
    },
    Close {
        id: tauri::ResourceId,
    },
}

#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum ReadFileStreamEventOutput {
    Open(tauri::ResourceId),
    Read(Vec<u8>),
    Close(()),
}

#[cfg(target_os = "android")]
impl TryFrom<ReadFileStreamEventOutput> for tauri::ipc::Response {
    type Error = Error;

    fn try_from(v: ReadFileStreamEventOutput) -> Result<tauri::ipc::Response> {
        match v {
            ReadFileStreamEventOutput::Open(id) => {
                 let id_bytes = convert_rid_to_bytes(id);
                 Ok(tauri::ipc::Response::new(id_bytes))
            },
            ReadFileStreamEventOutput::Read(bytes) => {
                Ok(tauri::ipc::Response::new(bytes))
            },
            ReadFileStreamEventOutput::Close(()) => {
                Ok(tauri::ipc::Response::new(Vec::new()))
            }
        }
    }
}

#[cfg(target_os = "android")]
pub struct FileChunkReader {
    file: std::fs::File,
    read_limit: Option<u64>,
    read: u64,
}

#[cfg(target_os = "android")]
impl FileChunkReader {

    pub fn new(file: std::fs::File, read_limit: Option<u64>) -> Self {
        Self {
            read_limit,
            read: 0,
            file,
        }
    }

    pub fn read_chunk(&mut self, len: u64) -> Result<Vec<u8>> {
        use std::io::Read as _;

        if self.read_limit.is_some_and(|l| l <= self.read) {
            return Ok(Vec::new())
        }

        let mut nlimit = len;
        if let Some(read_limit) = self.read_limit {
            nlimit = u64::min(nlimit, read_limit.saturating_sub(self.read));
        }

        let mut buf = Vec::with_capacity(usize::min(nlimit as usize, 2 * 1024 * 1024));

        let nread = self.file
            .by_ref()
            .take(nlimit)
            .read_to_end(&mut buf)?;

        self.read += nread as u64;

        Ok(buf)
    }
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum ReadTextFileLinesStreamEventInput {
    Open {
        uri: AfsUriOrFsPath,
        label: String,

        #[serde(rename = "maxLineByteLength")]
        max_line_len: u64,

        #[serde(rename = "ignoreBOM")]
        ignore_bom: bool,
    },
    Read {
        id: tauri::ResourceId,
        len: u64,
    },
    Close {
        id: tauri::ResourceId,
    },
}

#[cfg_attr(not(target_os = "android"), allow(unused))]
pub enum ReadTextFileLinesStreamEventOutput {
    Open(tauri::ResourceId),
    Read(Vec<u8>),
    Close(()),
}

#[cfg(target_os = "android")]
impl TryFrom<ReadTextFileLinesStreamEventOutput> for tauri::ipc::Response {
    type Error = Error;

    fn try_from(v: ReadTextFileLinesStreamEventOutput) -> Result<tauri::ipc::Response> {
        match v {
            ReadTextFileLinesStreamEventOutput::Open(id) => {
                 let id_bytes = convert_rid_to_bytes(id);
                 Ok(tauri::ipc::Response::new(id_bytes))
            },
            ReadTextFileLinesStreamEventOutput::Read(bytes) => {
                Ok(tauri::ipc::Response::new(bytes))
            },
            ReadTextFileLinesStreamEventOutput::Close(()) => {
                Ok(tauri::ipc::Response::new(Vec::new()))
            }
        }
    }
}

#[cfg(target_os = "android")]
pub struct FileTextLinesReader {
    file: std::io::BufReader<std::fs::File>,
    max_line_len: Option<std::num::NonZeroU64>,
    line_breaks: LineBreaks,
    bom: Option<&'static [u8]>,
    bom_handled: bool,
    read_limit: Option<u64>,
    read: u64
}

#[cfg(target_os = "android")]
impl FileTextLinesReader {

    pub fn new(
        file: std::fs::File,
        max_line_len: Option<std::num::NonZeroU64>,
        line_breaks: LineBreaks,
        bom: Option<&'static [u8]>,
        read_limit: Option<u64>,
    ) -> Self {

        Self {
            file: std::io::BufReader::new(file),
            max_line_len,
            line_breaks,
            bom,
            read_limit,
            bom_handled: false,
            read: 0
        }
    }

    /// この関数が返す bytes は以下の形式のレコードが連続したものであり、
    /// 各レコードが分断されることはない。
    /// 
    /// - err flag (u8, 0 = ok, 1 = err)
    /// - line break type (u8, 0 = null, 1 = '\n', 2 = '\r\n')
    /// - line bytes len (u64, big endian)
    /// - line bytes (variable bytes)
    /// 
    /// err flag が 0 の場合、正常にその行が読み込まれたことを指す。
    /// この場合、line bytes には BOM 処理されたテキストが格納される。
    /// 
    /// err flag が 1 の場合、その行でエラーが発生したことを示す。
    /// その場合、line bytes は utf-8 形式のエラーメッセージであり、
    /// この呼び出しでの最後の行となる。
    /// 
    /// エラー発生後の呼び出しの挙動は未定義。
    ///
    /// この関数は複数の行を先読みしてまとめて送信するため、
    /// 関数から直接エラーを返すのではなく、行単位でエラー情報を格納し、
    /// 対象行を明示的に読み込んだ際にエラーを発生させれるようにする。
    pub fn read_lines_framed(&mut self, threshold: u64) -> Result<Vec<u8>> {
        use std::io::Read as _;

        const ERR_FLAG_LEN: usize = 1;
        const LINE_BREAK_TYPE_LEN: usize = 1;
        const LINE_LEN_LEN: usize = 8;
        const HEADER_LEN: usize = ERR_FLAG_LEN + LINE_BREAK_TYPE_LEN + LINE_LEN_LEN;

        const FLAG_OK: u8 = 0;
        const FLAG_ERR: u8 = 1;
        const LINE_BREAK_NULL: u8 = 0;
        const LINE_BREAK_LF: u8 = 1;
        const LINE_BREAK_CRLF: u8 = 2;

        
        if self.read_limit.is_some_and(|l| l <= self.read) {
            return Ok(Vec::new())
        }

        let mut buf = Vec::with_capacity(usize::min(threshold as usize, 2 * 1024 * 1024));
        loop {
            let offset = buf.len();
            let header_offset = offset;
            let err_flag_offset = header_offset;
            let line_break_type_offset = err_flag_offset + ERR_FLAG_LEN;
            let line_len_offset = line_break_type_offset + LINE_BREAK_TYPE_LEN;
            let line_offset = line_len_offset + LINE_LEN_LEN;

            // header の場所を確保
            buf.extend_from_slice(&[0; HEADER_LEN]);

            let mut nlimit = u64::MAX;
            if let Some(read_limit) = self.read_limit {
                nlimit = u64::min(nlimit, read_limit.saturating_sub(self.read));
            } 
            if let Some(max_line_len) = self.max_line_len {
                // α は制限に含まない改行や BOM が取りうる最大の合計バイト数
                let mut alpha = self.line_breaks.lf.len() + self.line_breaks.cr.len();
                if !self.bom_handled {
                    alpha += self.bom.map(|b| b.len()).unwrap_or(0);
                }

                // 制限 + α のバイトを読み込み、
                // 制限を超えているかどうかで制限より大きいデータがあるかを判定する。
                let max_line_len = max_line_len.get().saturating_add(alpha as u64);

                nlimit = u64::min(nlimit, max_line_len);
            }

            // EOL ('\n', '\r\n') を検知するため '\n' が出るまで読み込む
            let nread = read_until_bytes(
                &mut self.file.by_ref().take(nlimit),
                &mut buf,
                &self.line_breaks.lf
            )?;
                    
            self.read += nread as u64;

            if nread == 0 || self.read_limit.is_some_and(|l| l <= self.read) {
                buf.truncate(offset);
                break
            }

            let mut line_len = nread;
            let mut line_break_type = LINE_BREAK_NULL;

            // 最後が EOL ('\n', '\r\n') で終わっていれば削除する。
            if self.line_breaks.lf.len() <= line_len && buf.ends_with(&self.line_breaks.lf) {
                buf.truncate(buf.len() - self.line_breaks.lf.len());
                line_len -= self.line_breaks.lf.len();
                line_break_type = LINE_BREAK_LF;
                if self.line_breaks.cr.len() <= line_len && buf.ends_with(&self.line_breaks.cr) {
                    buf.truncate(buf.len() - self.line_breaks.cr.len());
                    line_len -= self.line_breaks.cr.len();
                    line_break_type = LINE_BREAK_CRLF;
                }
            }
            // BOM をまだ処理していない場合、必要であれば削除する
            if !self.bom_handled {
                self.bom_handled = true;
                if let Some(bom) = self.bom {
                    if buf[line_offset..].starts_with(bom) {
                        buf.drain(line_offset..line_offset + bom.len());
                        line_len -= bom.len();
                    }
                }
            }

            // エラーとなるかの確認
            let checked = (|| -> Result<()> {
                if self.max_line_len.is_some_and(|i| i.get() < line_len as u64) {
                    return Err(Error::with("line length limit exceeded"));
                }
                Ok(())
            })();
                        
            if let Err(err) = checked {
                let err_msg_bytes = err.to_string().into_bytes();

                // header を設定
                buf[err_flag_offset] = FLAG_ERR;
                buf[line_break_type_offset] = LINE_BREAK_NULL;
                buf[line_len_offset..(line_len_offset + LINE_LEN_LEN)]
                    .copy_from_slice(&u64::to_be_bytes(err_msg_bytes.len() as u64));

                // データをエラーメッセージに差し替える
                buf.truncate(line_offset);
                buf.extend_from_slice(&err_msg_bytes);
                break
            }
            else {
                // header を設定
                buf[err_flag_offset] = FLAG_OK;
                buf[line_break_type_offset] = line_break_type;
                buf[line_len_offset..(line_len_offset + LINE_LEN_LEN)]
                    .copy_from_slice(&u64::to_be_bytes(line_len as u64));

                if threshold <= (buf.len() as u64) {
                    break
                }
            }
        }

        Ok(buf)
    }
}

/// label は `(new TextDecoder(encoding)).encoding` などで正規化された小文字のテキスト
#[cfg(target_os = "android")]
pub fn bom_for_encoding_label(label: &str) -> Option<&'static [u8]> {
    // WEB 標準で定義されているエンコーディングのうち
    // UTF-8, UTF-16 LE/BE のみが BOM を持つ。
    match label {
        "utf-8" => Some(b"\xEF\xBB\xBF"),
        "utf-16le" => Some(b"\xFF\xFE"),
        "utf-16be" => Some(b"\xFE\xFF"),
        _ => None
    }
}

#[cfg(target_os = "android")]
pub struct LineBreaks {
    pub lf: &'static [u8],
    pub cr: &'static [u8],
}

/// label は `(new TextDecoder(encoding)).encoding` などで正規化された小文字のテキスト
#[cfg(target_os = "android")]
pub fn line_breaks_for_encoding_label(label: &str) -> LineBreaks {
    // WEB 標準で定義されているエンコーディングのうち
    // UTF-16 LE/BE, ISO 2022-JP が ASCII 互換ではない。
    // ただし、ISO 2022-JP は ASCII と同じ改行コードである。
    match label {
        "utf-16le" => LineBreaks {
            lf: &[0x0A, 0x00],
            cr: &[0x0D, 0x00],
        },
        "utf-16be" => LineBreaks {
            lf: &[0x00, 0x0A],
            cr: &[0x00, 0x0D],
        },
        _ => LineBreaks {
            lf: &[b'\n'],
            cr: &[b'\r'],
        },
    }
}

#[cfg(target_os = "android")]
fn read_until_bytes(
    r: &mut impl std::io::BufRead,
    buf: &mut Vec<u8>,
    bytes: &[u8]
) -> Result<usize> {

    let last_byte = *bytes.last().ok_or_else(|| Error::with("invalid empty bytes"))?;

    if bytes.len() == 1 {
        return Ok(r.read_until(last_byte, buf)?);
    }

    let mut total_n = 0;
    loop {
        let n = r.read_until(last_byte, buf)?;
        total_n += n;

        if n == 0 || buf.ends_with(bytes) {
            return Ok(total_n)
        }
    }
}

#[cfg(target_os = "android")]
pub fn convert_rid_to_bytes(rid: tauri::ResourceId) -> Vec<u8> {
    rid.to_be_bytes().to_vec()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PickerInitialLocation {
    Any {
        uri: FileUri,
    },
    VolumeTop {
        #[serde(rename = "volumeId")]
        volume_id: Option<String>,
    },
    PublicDir {
        #[serde(rename = "baseDir")]
        base_dir: PublicDir,

        #[serde(rename = "relativePath")]
        relative_path: Option<String>,

        #[serde(rename = "volumeId")]
        volume_id: Option<String>,
    },
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone, Copy)]
pub enum FilePickerType {
    FilePicker,
    Gallery
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PublicImageOrGeneralPurposeDir {
    Image(PublicImageDir),
    GeneralPurpose(PublicGeneralPurposeDir),
}

impl From<PublicImageOrGeneralPurposeDir> for PublicDir {

    fn from(value: PublicImageOrGeneralPurposeDir) -> Self {
        match value {
            PublicImageOrGeneralPurposeDir::Image(d) => d.into(),
            PublicImageOrGeneralPurposeDir::GeneralPurpose(d) => d.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PublicVideoOrGeneralPurposeDir {
    Video(PublicVideoDir),
    GeneralPurpose(PublicGeneralPurposeDir),
}

impl From<PublicVideoOrGeneralPurposeDir> for PublicDir {

    fn from(value: PublicVideoOrGeneralPurposeDir) -> Self {
        match value {
            PublicVideoOrGeneralPurposeDir::Video(d) => d.into(),
            PublicVideoOrGeneralPurposeDir::GeneralPurpose(d) => d.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum PublicAudioOrGeneralPurposeDir {
    Audio(PublicAudioDir),
    GeneralPurpose(PublicGeneralPurposeDir),
}

impl From<PublicAudioOrGeneralPurposeDir> for PublicDir {

    fn from(value: PublicAudioOrGeneralPurposeDir) -> Self {
        match value {
            PublicAudioOrGeneralPurposeDir::Audio(d) => d.into(),
            PublicAudioOrGeneralPurposeDir::GeneralPurpose(d) => d.into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum AfsUriOrFsPath {
    AfsUri(FileUri),
    FsPath(tauri_plugin_fs::FilePath),
}

impl TryFrom<AfsUriOrFsPath> for FileUri {
    type Error = Error;

    fn try_from(value: AfsUriOrFsPath) -> Result<Self> {
        // Content URI かパスのみを受け入れる。
        // File scheme URI は受け入れない。

        match value {
            AfsUriOrFsPath::AfsUri(uri) => {
                uri.require_content_uri()?;
                Ok(uri)
            },
            AfsUriOrFsPath::FsPath(path) => {
                match path {
                    tauri_plugin_fs::FilePath::Path(path) => Ok(FileUri::from_path(path)),
                    tauri_plugin_fs::FilePath::Url(url) => {
                        if url.scheme() != "content" {
                            return Err(Error::invalid_uri_scheme(url))
                        }
                        Ok(FileUri::from_uri(url))
                    }
                }
            },
        }
    }
}

impl From<AfsUriOrFsPath> for tauri_plugin_fs::FilePath {

    fn from(value: AfsUriOrFsPath) -> Self {
        match value {
            AfsUriOrFsPath::AfsUri(uri) => uri.into(),
            AfsUriOrFsPath::FsPath(path) => path,
        }
    }
}

// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/src/commands.rs#L1090
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
#[cfg(target_os = "android")]
pub fn validate_path_permission<R: tauri::Runtime>(
    path: impl AsRef<std::path::Path>,
    app: &tauri::AppHandle<R>,
    cmd_scope: &tauri::ipc::CommandScope<Scope>,
    global_scope: &tauri::ipc::GlobalScope<Scope>,
) -> Result<()> {

    let path = path.as_ref();
    let require_literal_leading_dot = true;

    let scope = tauri::scope::fs::Scope::new(
        app,
        &tauri::utils::config::FsScope::Scope {
            allow: global_scope
                .allows()
                .iter()
                .filter_map(|e| e.path.clone())
                .chain(cmd_scope.allows().iter().filter_map(|e| e.path.clone()))
                .collect(),

            deny: global_scope
                .denies()
                .iter()
                .filter_map(|e| e.path.clone())
                .chain(cmd_scope.denies().iter().filter_map(|e| e.path.clone()))
                .collect(),

            require_literal_leading_dot: Some(require_literal_leading_dot),
        },
    )?;

    if !is_forbidden(&scope, &path, require_literal_leading_dot) && scope.is_allowed(&path) {
        return Ok(())
    }
    
    if cfg!(debug_assertions) {
        Err(Error::with(format!(
            "forbidden path: {}, maybe it is not allowed on the scope configuration in your capability file",
            path.display()
        )))
    }
    else {
        Err(Error::with(format!(
            "forbidden path: {}", 
            path.display()
        )))
    }
}

// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/src/commands.rs#L1151
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
#[cfg(target_os = "android")]
fn is_forbidden<P: AsRef<std::path::Path>>(
    scope: &tauri::fs::Scope,
    path: P,
    require_literal_leading_dot: bool,
) -> bool {

    let path = path.as_ref();
    let path = if path.is_symlink() {
        match std::fs::read_link(path) {
            Ok(p) => p,
            Err(_) => return false,
        }
    } else {
        path.to_path_buf()
    };
    let path = if !path.exists() {
        crate::Result::Ok(path)
    } else {
        std::fs::canonicalize(path).map_err(Into::into)
    };

    if let Ok(path) = path {
        let path: std::path::PathBuf = path.components().collect();
        scope.forbidden_patterns().iter().any(|p| {
            p.matches_path_with(
                &path,
                glob::MatchOptions {
                    // this is needed so `/dir/*` doesn't match files within subdirectories such as `/dir/subdir/file.txt`
                    // see: <https://github.com/tauri-apps/tauri/security/advisories/GHSA-6mv3-wm7j-h4w5>
                    require_literal_separator: true,
                    require_literal_leading_dot,
                    ..Default::default()
                },
            )
        })
    } else {
        false
    }
}

// Based on code from tauri-plugin-fs crate
//
// Source:
// - https://github.com/tauri-apps/plugins-workspace/blob/3d0d2e041bbad9766aebecaeba291a28d8d7bf5c/plugins/fs/src/lib.rs#L347
// - Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// - Licensed under the MIT License or the Apache 2.0 License
impl tauri::ipc::ScopeObject for Scope {
    type Error = Error;

    fn deserialize<R: tauri::Runtime>(
        app: &tauri::AppHandle<R>,
        raw: tauri::utils::acl::Value
    ) -> Result<Self> {
        
        let path = serde_json::from_value(raw.into()).map(|raw| match raw {
            ScopeSchema::Value(path) => path,
            ScopeSchema::Object { path } => path,
        })?;

        match app.path().parse(path) {
            Ok(path) => Ok(Self { path: Some(path) }),
            Err(err) => Err(err.into()),
        }
    }
}