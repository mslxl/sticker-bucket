use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
    time::Duration,
};

use image::{DynamicImage, ImageFormat, Rgba, RgbaImage, imageops};
use memelith_clip::{ClipModel, ExecutionPolicy};
use memelith_core::{COLLECTOR_DUPLICATE_MAX_COSINE_DISTANCE, MemeDatabase};
use reqwest::blocking::{Client, multipart};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;

const TELEGRAM_API: &str = "https://api.telegram.org";
const UPDATE_TIMEOUT_SECONDS: i64 = 2;
const HTTP_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_SIMILAR_PER_PAGE: usize = 3;
const PREVIEW_TILE_SIZE: u32 = 480;
const PREVIEW_GUTTER: u32 = 16;

#[derive(Debug, Error)]
enum TelegramError {
    #[error("Telegram request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Telegram returned an error: {0}")]
    Api(String),

    #[error("Telegram response did not contain a result")]
    MissingResult,

    #[error("Telegram response contained invalid data: {0}")]
    Json(#[from] serde_json::Error),

    #[error("image operation failed: {0}")]
    Image(#[from] image::ImageError),

    #[error("file operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("Memelith database operation failed: {0}")]
    Database(#[from] memelith_core::Error),

    #[error("Telegram file response did not include a file path")]
    MissingFilePath,

    #[error("invalid callback data")]
    InvalidCallback,
}

pub struct TelegramBotHandle {
    stop: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl TelegramBotHandle {
    pub fn start(
        storage_root: PathBuf,
        token: String,
        model_directory: PathBuf,
    ) -> std::io::Result<Self> {
        let (stop, stop_receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("memelith-telegram".to_owned())
            .spawn(move || {
                if let Err(error) = run_bot(storage_root, token, model_directory, stop_receiver) {
                    eprintln!("Memelith Telegram bot stopped: {error}");
                }
            })?;
        Ok(Self {
            stop: Some(stop),
            thread: Some(thread),
        })
    }

    pub fn stop(mut self) {
        self.request_stop();
        self.join();
    }

    fn request_stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }

    fn join(&mut self) {
        if let Some(thread) = self.thread.take()
            && thread.thread().id() != thread::current().id()
            && thread.join().is_err()
        {
            eprintln!("Memelith Telegram bot thread panicked");
        }
    }
}

impl Drop for TelegramBotHandle {
    fn drop(&mut self) {
        self.request_stop();
        self.join();
    }
}

struct BotState {
    api: TelegramApi,
    database: Mutex<MemeDatabase>,
    storage_root: PathBuf,
    pending: Mutex<HashMap<Uuid, PendingOperation>>,
}

#[derive(Clone)]
struct PendingOperation {
    id: Uuid,
    chat_id: i64,
    message_id: i64,
    source_path: PathBuf,
    similar_paths: Vec<PathBuf>,
    page: usize,
}

#[derive(Clone)]
struct TelegramApi {
    client: Client,
    base_url: String,
}

impl TelegramApi {
    fn new(token: &str) -> Result<Self, TelegramError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(TelegramError::Api("Telegram Bot token is empty".to_owned()));
        }
        let client = Client::builder().timeout(HTTP_TIMEOUT).build()?;
        Ok(Self {
            client,
            base_url: format!("{TELEGRAM_API}/bot{token}"),
        })
    }

    fn call<T: DeserializeOwned>(
        &self,
        method: &str,
        parameters: impl Serialize,
    ) -> Result<T, TelegramError> {
        let response = self
            .client
            .post(format!("{}/{method}", self.base_url))
            .json(&parameters)
            .send()?;
        let response = response.error_for_status()?;
        let body = response.json::<ApiResponse<T>>()?;
        if !body.ok {
            return Err(TelegramError::Api(
                body.description
                    .unwrap_or_else(|| "unknown Telegram error".to_owned()),
            ));
        }
        body.result.ok_or(TelegramError::MissingResult)
    }

    fn get_updates(&self, offset: i64) -> Result<Vec<Update>, TelegramError> {
        self.call(
            "getUpdates",
            json!({
                "offset": offset,
                "timeout": UPDATE_TIMEOUT_SECONDS,
                "allowed_updates": ["message", "callback_query"],
            }),
        )
    }

    fn get_file(&self, file_id: &str) -> Result<TelegramFile, TelegramError> {
        self.call("getFile", json!({ "file_id": file_id }))
    }

    fn download_file(&self, file_path: &str) -> Result<Vec<u8>, TelegramError> {
        let token_url = self
            .base_url
            .strip_prefix(TELEGRAM_API)
            .ok_or_else(|| TelegramError::Api("invalid Telegram API URL".to_owned()))?;
        let url = format!("{TELEGRAM_API}/file{token_url}/{file_path}");
        Ok(self
            .client
            .get(url)
            .send()?
            .error_for_status()?
            .bytes()?
            .to_vec())
    }

    fn send_message(&self, chat_id: i64, text: &str) -> Result<Message, TelegramError> {
        self.call("sendMessage", json!({"chat_id": chat_id, "text": text}))
    }

    fn send_preview(
        &self,
        chat_id: i64,
        image: Vec<u8>,
        caption: &str,
        markup: &InlineKeyboardMarkup,
    ) -> Result<Message, TelegramError> {
        let part = multipart::Part::bytes(image)
            .file_name("memelith-preview.png")
            .mime_str("image/png")?;
        let form = multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .text("caption", caption.to_owned())
            .text("reply_markup", serde_json::to_string(markup)?)
            .part("photo", part);
        self.multipart_call("sendPhoto", form)
    }

    fn edit_preview(
        &self,
        chat_id: i64,
        message_id: i64,
        image: Vec<u8>,
        caption: &str,
        markup: &InlineKeyboardMarkup,
    ) -> Result<Message, TelegramError> {
        let part = multipart::Part::bytes(image)
            .file_name("memelith-preview.png")
            .mime_str("image/png")?;
        let media = json!({
            "type": "photo",
            "media": "attach://memelith-preview.png",
            "caption": caption,
        });
        let form = multipart::Form::new()
            .text("chat_id", chat_id.to_string())
            .text("message_id", message_id.to_string())
            .text("media", serde_json::to_string(&media)?)
            .text("reply_markup", serde_json::to_string(markup)?)
            .part("memelith-preview.png", part);
        self.multipart_call("editMessageMedia", form)
    }

    fn answer_callback(
        &self,
        callback_id: &str,
        text: Option<&str>,
    ) -> Result<bool, TelegramError> {
        self.call(
            "answerCallbackQuery",
            json!({"callback_query_id": callback_id, "text": text}),
        )
    }

    fn delete_message(&self, chat_id: i64, message_id: i64) -> Result<bool, TelegramError> {
        self.call(
            "deleteMessage",
            json!({"chat_id": chat_id, "message_id": message_id}),
        )
    }

    fn multipart_call<T: DeserializeOwned>(
        &self,
        method: &str,
        form: multipart::Form,
    ) -> Result<T, TelegramError> {
        let response = self
            .client
            .post(format!("{}/{method}", self.base_url))
            .multipart(form)
            .send()?;
        let response = response.error_for_status()?;
        let body = response.json::<ApiResponse<T>>()?;
        if !body.ok {
            return Err(TelegramError::Api(
                body.description
                    .unwrap_or_else(|| "unknown Telegram error".to_owned()),
            ));
        }
        body.result.ok_or(TelegramError::MissingResult)
    }
}

fn run_bot(
    storage_root: PathBuf,
    token: String,
    model_directory: PathBuf,
    stop_receiver: mpsc::Receiver<()>,
) -> Result<(), TelegramError> {
    let api = TelegramApi::new(&token)?;
    let model = ClipModel::load(model_directory, ExecutionPolicy::Auto)
        .map_err(|error| TelegramError::Api(format!("failed to load CLIP model: {error}")))?;
    let database = MemeDatabase::open(&storage_root, model)?;
    let state = Arc::new(BotState {
        api,
        database: Mutex::new(database),
        storage_root,
        pending: Mutex::new(HashMap::new()),
    });

    let mut offset = 0_i64;
    loop {
        if stop_receiver.try_recv().is_ok() {
            break;
        }
        match state.api.get_updates(offset) {
            Ok(updates) => {
                for update in updates {
                    offset = update.update_id.saturating_add(1);
                    let state = Arc::clone(&state);
                    thread::Builder::new()
                        .name("memelith-telegram-update".to_owned())
                        .spawn(move || {
                            if let Err(error) = handle_update(state, update) {
                                eprintln!("Memelith Telegram update failed: {error}");
                            }
                        })
                        .map_err(|error| TelegramError::Io(std::io::Error::other(error)))?;
                }
            }
            Err(error) => {
                eprintln!("Memelith Telegram polling failed: {error}");
                if stop_receiver.recv_timeout(Duration::from_secs(1)).is_ok() {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn handle_update(state: Arc<BotState>, update: Update) -> Result<(), TelegramError> {
    if let Some(callback) = update.callback_query {
        return handle_callback(state, callback);
    }
    if let Some(message) = update.message {
        let Some(photo) = largest_photo(&message.photo) else {
            return Ok(());
        };
        let chat_id = message.chat.id;
        if let Err(error) = handle_photo(Arc::clone(&state), message, photo) {
            log_api_result(
                "send image failure message",
                state
                    .api
                    .send_message(chat_id, &format!("图片处理失败：{error}")),
            );
            return Err(error);
        }
    }
    Ok(())
}

fn handle_photo(
    state: Arc<BotState>,
    message: Message,
    photo: PhotoSize,
) -> Result<(), TelegramError> {
    let file = state.api.get_file(&photo.file_id)?;
    let file_path = file.file_path.ok_or(TelegramError::MissingFilePath)?;
    let bytes = state.api.download_file(&file_path)?;
    let incoming_directory = state.storage_root.join(".telegram/incoming");
    fs::create_dir_all(&incoming_directory)?;
    let source_path = incoming_directory.join(format!("{}.image", Uuid::new_v4()));
    fs::write(&source_path, bytes)?;

    let duplicate_result = {
        let mut database = state
            .database
            .lock()
            .map_err(|_| TelegramError::Api("database lock was poisoned".to_owned()))?;
        database.find_image_duplicates(&source_path, COLLECTOR_DUPLICATE_MAX_COSINE_DISTANCE)
    };
    let duplicates = match duplicate_result {
        Ok(duplicates) => duplicates,
        Err(error) => {
            remove_temp_file(&source_path);
            return Err(error.into());
        }
    };

    if duplicates.is_empty() {
        let mut database = state
            .database
            .lock()
            .map_err(|_| TelegramError::Api("database lock was poisoned".to_owned()))?;
        database.collect_image(&source_path)?;
        log_api_result(
            "delete collected source message",
            state
                .api
                .delete_message(message.chat.id, message.message_id),
        );
        fs::remove_file(source_path)?;
        return Ok(());
    }

    let similar_paths = match duplicates
        .iter()
        .map(|duplicate| {
            MemeDatabase::resolve_media_path_from_root(
                &state.storage_root,
                &duplicate.relative_path,
            )
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(paths) => paths,
        Err(error) => {
            remove_temp_file(&source_path);
            return Err(error.into());
        }
    };
    let operation_id = Uuid::new_v4();
    let pending = PendingOperation {
        id: operation_id,
        chat_id: message.chat.id,
        message_id: 0,
        source_path,
        similar_paths,
        page: 0,
    };
    let preview = match render_preview(&pending) {
        Ok(preview) => preview,
        Err(error) => {
            remove_temp_file(&pending.source_path);
            return Err(error);
        }
    };
    let caption = preview_caption(&pending, duplicates.len());
    let markup = preview_markup(&pending);
    let sent = state
        .api
        .send_preview(message.chat.id, preview, &caption, &markup)
        .inspect_err(|_| {
            remove_temp_file(&pending.source_path);
        })?;
    let mut pending = pending;
    pending.message_id = sent.message_id;
    state
        .pending
        .lock()
        .map_err(|_| TelegramError::Api("pending operation lock was poisoned".to_owned()))?
        .insert(operation_id, pending);
    log_api_result(
        "delete duplicate source message",
        state
            .api
            .delete_message(message.chat.id, message.message_id),
    );
    Ok(())
}

fn handle_callback(state: Arc<BotState>, callback: CallbackQuery) -> Result<(), TelegramError> {
    log_api_result(
        "answer callback query",
        state.api.answer_callback(&callback.id, None),
    );
    let data = callback
        .data
        .as_deref()
        .ok_or(TelegramError::InvalidCallback)?;
    let (operation_id, action, page) = parse_callback(data)?;

    if action != "page" && action != "keep" && action != "discard" {
        return Err(TelegramError::InvalidCallback);
    }

    if action == "page" {
        let pending = {
            let mut operations = state.pending.lock().map_err(|_| {
                TelegramError::Api("pending operation lock was poisoned".to_owned())
            })?;
            let operation = operations
                .get_mut(&operation_id)
                .ok_or(TelegramError::InvalidCallback)?;
            let page = page.ok_or(TelegramError::InvalidCallback)?;
            operation.page = page.min(page_count(operation.similar_paths.len()).saturating_sub(1));
            operation.clone()
        };
        let preview = render_preview(&pending)?;
        let caption = preview_caption(&pending, pending.similar_paths.len());
        let markup = preview_markup(&pending);
        state.api.edit_preview(
            pending.chat_id,
            pending.message_id,
            preview,
            &caption,
            &markup,
        )?;
        return Ok(());
    }

    let pending = state
        .pending
        .lock()
        .map_err(|_| TelegramError::Api("pending operation lock was poisoned".to_owned()))?
        .remove(&operation_id)
        .ok_or(TelegramError::InvalidCallback)?;
    if action == "keep" {
        let mut database = state
            .database
            .lock()
            .map_err(|_| TelegramError::Api("database lock was poisoned".to_owned()))?;
        if let Err(error) = database.collect_image(&pending.source_path) {
            drop(database);
            state
                .pending
                .lock()
                .map_err(|_| TelegramError::Api("pending operation lock was poisoned".to_owned()))?
                .insert(pending.id, pending.clone());
            log_api_result(
                "send Collector failure message",
                state
                    .api
                    .send_message(pending.chat_id, &format!("仍然添加失败：{error}")),
            );
            return Err(error.into());
        }
    } else if action != "discard" {
        return Err(TelegramError::InvalidCallback);
    }
    remove_temp_file(&pending.source_path);
    log_api_result(
        "delete duplicate confirmation message",
        state
            .api
            .delete_message(pending.chat_id, pending.message_id),
    );
    Ok(())
}

fn parse_callback(data: &str) -> Result<(Uuid, &str, Option<usize>), TelegramError> {
    let mut fields = data.split(':');
    if fields.next() != Some("memelith") {
        return Err(TelegramError::InvalidCallback);
    }
    let operation_id = fields
        .next()
        .ok_or(TelegramError::InvalidCallback)?
        .parse()
        .map_err(|_| TelegramError::InvalidCallback)?;
    let action = fields.next().ok_or(TelegramError::InvalidCallback)?;
    let page = fields
        .next()
        .map(|value| value.parse().map_err(|_| TelegramError::InvalidCallback))
        .transpose()?;
    if fields.next().is_some() {
        return Err(TelegramError::InvalidCallback);
    }
    Ok((operation_id, action, page))
}

fn render_preview(operation: &PendingOperation) -> Result<Vec<u8>, TelegramError> {
    let paths = preview_paths(operation);
    let mut canvas = RgbaImage::from_pixel(
        PREVIEW_TILE_SIZE * 2 + PREVIEW_GUTTER * 3,
        PREVIEW_TILE_SIZE * 2 + PREVIEW_GUTTER * 3,
        Rgba([245, 245, 247, 255]),
    );
    for (index, path) in paths.iter().enumerate() {
        let image = image::open(path)?;
        let thumbnail = image.thumbnail(PREVIEW_TILE_SIZE, PREVIEW_TILE_SIZE);
        let x = PREVIEW_GUTTER
            + (index as u32 % 2) * (PREVIEW_TILE_SIZE + PREVIEW_GUTTER)
            + (PREVIEW_TILE_SIZE - thumbnail.width()) / 2;
        let y = PREVIEW_GUTTER
            + (index as u32 / 2) * (PREVIEW_TILE_SIZE + PREVIEW_GUTTER)
            + (PREVIEW_TILE_SIZE - thumbnail.height()) / 2;
        imageops::overlay(
            &mut canvas,
            &thumbnail.to_rgba8(),
            i64::from(x),
            i64::from(y),
        );
    }
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(canvas).write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

fn preview_paths(operation: &PendingOperation) -> Vec<PathBuf> {
    let start = operation.page.saturating_mul(MAX_SIMILAR_PER_PAGE);
    let end = (start + MAX_SIMILAR_PER_PAGE).min(operation.similar_paths.len());
    let mut paths = Vec::with_capacity(end.saturating_sub(start) + 1);
    paths.push(operation.source_path.clone());
    paths.extend(operation.similar_paths[start..end].iter().cloned());
    paths
}

fn page_count(similar_count: usize) -> usize {
    similar_count.div_ceil(MAX_SIMILAR_PER_PAGE).max(1)
}

fn preview_caption(operation: &PendingOperation, total: usize) -> String {
    format!(
        "检测到 {total} 张相似图片。第 {} / {} 页，请选择是否仍然添加到 Collector。",
        operation.page + 1,
        page_count(operation.similar_paths.len())
    )
}

fn preview_markup(operation: &PendingOperation) -> InlineKeyboardMarkup {
    let pages = page_count(operation.similar_paths.len());
    let mut rows = Vec::new();
    if pages > 1 {
        let mut navigation = Vec::new();
        if operation.page > 0 {
            navigation.push(InlineKeyboardButton {
                text: "上一页".to_owned(),
                callback_data: format!("memelith:{}:page:{}", operation.id, operation.page - 1),
            });
        }
        if operation.page + 1 < pages {
            navigation.push(InlineKeyboardButton {
                text: "下一页".to_owned(),
                callback_data: format!("memelith:{}:page:{}", operation.id, operation.page + 1),
            });
        }
        rows.push(navigation);
    }
    rows.push(vec![
        InlineKeyboardButton {
            text: "仍然添加".to_owned(),
            callback_data: format!("memelith:{}:keep", operation.id),
        },
        InlineKeyboardButton {
            text: "放弃".to_owned(),
            callback_data: format!("memelith:{}:discard", operation.id),
        },
    ]);
    InlineKeyboardMarkup {
        inline_keyboard: rows,
    }
}

fn largest_photo(photos: &[PhotoSize]) -> Option<PhotoSize> {
    photos.iter().cloned().max_by_key(|photo| {
        (
            u64::from(photo.width).saturating_mul(u64::from(photo.height)),
            photo.file_size.unwrap_or_default(),
        )
    })
}

fn log_api_result<T>(operation: &str, result: Result<T, TelegramError>) {
    if let Err(error) = result {
        eprintln!("Memelith Telegram failed to {operation}: {error}");
    }
}

fn remove_temp_file(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != std::io::ErrorKind::NotFound
    {
        eprintln!(
            "Memelith Telegram failed to remove {}: {error}",
            path.display()
        );
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Update {
    update_id: i64,
    message: Option<Message>,
    callback_query: Option<CallbackQuery>,
}

#[derive(Clone, Debug, Deserialize)]
struct Message {
    message_id: i64,
    chat: Chat,
    #[serde(default)]
    photo: Vec<PhotoSize>,
}

#[derive(Clone, Debug, Deserialize)]
struct Chat {
    id: i64,
}

#[derive(Clone, Debug, Deserialize)]
struct PhotoSize {
    file_id: String,
    width: u32,
    height: u32,
    file_size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TelegramFile {
    file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    id: String,
    data: Option<String>,
}

#[derive(Debug, Serialize)]
struct InlineKeyboardMarkup {
    inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

#[derive(Debug, Serialize)]
struct InlineKeyboardButton {
    text: String,
    callback_data: String,
}
