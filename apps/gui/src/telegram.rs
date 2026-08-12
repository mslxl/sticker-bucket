use std::{
    collections::{HashMap, HashSet},
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

    #[error("callback is not authorized for this operation")]
    UnauthorizedCallback,

    #[error("Telegram message did not include a sender")]
    MissingSender,
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
    pending_deletions: Mutex<HashSet<(i64, i64)>>,
    workers: Mutex<Vec<JoinHandle<()>>>,
}

#[derive(Clone)]
struct PendingOperation {
    id: Uuid,
    chat_id: i64,
    message_id: i64,
    user_id: i64,
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
        pending_deletions: Mutex::new(HashSet::new()),
        workers: Mutex::new(Vec::new()),
    });

    let mut offset = 0_i64;
    loop {
        if stop_receiver.try_recv().is_ok() {
            break;
        }
        reap_finished_workers(&state);
        retry_pending_deletions(&state);
        match state.api.get_updates(offset) {
            Ok(updates) => {
                for update in updates {
                    offset = update.update_id.saturating_add(1);
                    let worker_state = Arc::clone(&state);
                    let worker = thread::Builder::new()
                        .name("memelith-telegram-update".to_owned())
                        .spawn(move || {
                            if let Err(error) = handle_update(worker_state, update) {
                                eprintln!("Memelith Telegram update failed: {error}");
                            }
                        })
                        .map_err(|error| TelegramError::Io(std::io::Error::other(error)));
                    let worker = match worker {
                        Ok(worker) => worker,
                        Err(error) => {
                            shutdown_state(&state);
                            return Err(error);
                        }
                    };
                    let mut workers = match state.workers.lock() {
                        Ok(workers) => workers,
                        Err(_) => {
                            if worker.join().is_err() {
                                eprintln!("Memelith Telegram update worker panicked");
                            }
                            shutdown_state(&state);
                            return Err(TelegramError::Api("worker lock was poisoned".to_owned()));
                        }
                    };
                    workers.push(worker);
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
    shutdown_state(&state);
    Ok(())
}

fn shutdown_state(state: &Arc<BotState>) {
    let workers = match state.workers.lock() {
        Ok(mut workers) => std::mem::take(&mut *workers),
        Err(_) => {
            eprintln!("Memelith Telegram failed to acquire worker lock during shutdown");
            Vec::new()
        }
    };
    for worker in workers {
        if worker.join().is_err() {
            eprintln!("Memelith Telegram update worker panicked during shutdown");
        }
    }

    match state.pending.lock() {
        Ok(mut pending) => {
            for operation in pending.drain().map(|(_, operation)| operation) {
                remove_temp_file(&operation.source_path);
            }
        }
        Err(_) => {
            eprintln!("Memelith Telegram failed to acquire pending lock during shutdown");
        }
    }
}

fn reap_finished_workers(state: &Arc<BotState>) {
    let mut finished = Vec::new();
    match state.workers.lock() {
        Ok(mut workers) => {
            let mut active = Vec::with_capacity(workers.len());
            for worker in workers.drain(..) {
                if worker.is_finished() {
                    finished.push(worker);
                } else {
                    active.push(worker);
                }
            }
            *workers = active;
        }
        Err(_) => {
            eprintln!("Memelith Telegram failed to acquire worker lock while reaping");
            return;
        }
    }
    for worker in finished {
        if worker.join().is_err() {
            eprintln!("Memelith Telegram update worker panicked");
        }
    }
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
    let sender = message.from.as_ref().ok_or(TelegramError::MissingSender)?;
    let file = state.api.get_file(&photo.file_id)?;
    let file_path = file.file_path.ok_or(TelegramError::MissingFilePath)?;
    let bytes = state.api.download_file(&file_path)?;
    let incoming_directory = state.storage_root.join(".telegram/incoming");
    fs::create_dir_all(&incoming_directory)?;
    let source_path = incoming_directory.join(format!("{}.image", Uuid::new_v4()));
    let mut source_file = TempFileGuard::new(source_path);
    fs::write(source_file.path(), bytes)?;

    let duplicate_result = {
        let mut database = state
            .database
            .lock()
            .map_err(|_| TelegramError::Api("database lock was poisoned".to_owned()))?;
        database.find_image_duplicates(source_file.path(), COLLECTOR_DUPLICATE_MAX_COSINE_DISTANCE)
    };
    let duplicates = duplicate_result?;

    if duplicates.is_empty() {
        let mut database = state
            .database
            .lock()
            .map_err(|_| TelegramError::Api("database lock was poisoned".to_owned()))?;
        database.collect_image(source_file.path())?;
        drop(database);
        if !delete_message_or_queue(&state, message.chat.id, message.message_id)? {
            log_api_result(
                "send source deletion warning",
                state.api.send_message(
                    message.chat.id,
                    "图片已添加，但原消息暂时无法删除，机器人会自动重试。",
                ),
            );
        }
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
            return Err(error.into());
        }
    };
    let operation_id = Uuid::new_v4();
    let pending = PendingOperation {
        id: operation_id,
        chat_id: message.chat.id,
        message_id: 0,
        user_id: sender.id,
        source_path: source_file.path().to_owned(),
        similar_paths,
        page: 0,
    };
    let preview = render_preview(&pending)?;
    let caption = preview_caption(&pending, duplicates.len());
    let markup = preview_markup(&pending);
    let sent = state
        .api
        .send_preview(message.chat.id, preview, &caption, &markup)?;
    let mut pending = pending;
    pending.message_id = sent.message_id;
    let pending_result = state
        .pending
        .lock()
        .map_err(|_| TelegramError::Api("pending operation lock was poisoned".to_owned()));
    let mut operations = match pending_result {
        Ok(operations) => operations,
        Err(error) => {
            if let Err(delete_error) =
                delete_message_or_queue(&state, sent.chat.id, sent.message_id)
            {
                eprintln!(
                    "Memelith Telegram failed to clean up orphaned confirmation message: {delete_error}"
                );
            }
            return Err(error);
        }
    };
    operations.insert(operation_id, pending);
    drop(operations);
    source_file.disarm();
    delete_message_or_queue(&state, message.chat.id, message.message_id)?;
    Ok(())
}

fn handle_callback(state: Arc<BotState>, callback: CallbackQuery) -> Result<(), TelegramError> {
    let data = callback
        .data
        .as_deref()
        .ok_or(TelegramError::InvalidCallback)?;
    let (operation_id, action, page) = parse_callback(data)?;

    if action != "page" && action != "keep" && action != "discard" {
        return Err(TelegramError::InvalidCallback);
    }
    if action != "page" && page.is_some() {
        return Err(TelegramError::InvalidCallback);
    }

    let callback_message = match callback.message.as_ref() {
        Some(message) => message,
        None => {
            log_api_result(
                "answer callback query without message",
                state
                    .api
                    .answer_callback(&callback.id, Some("此按钮已失效。")),
            );
            return Err(TelegramError::UnauthorizedCallback);
        }
    };
    let pending_snapshot = {
        let operations = state
            .pending
            .lock()
            .map_err(|_| TelegramError::Api("pending operation lock was poisoned".to_owned()))?;
        operations
            .get(&operation_id)
            .cloned()
            .ok_or(TelegramError::InvalidCallback)?
    };
    if callback.from.id != pending_snapshot.user_id
        || callback_message.chat.id != pending_snapshot.chat_id
        || callback_message.message_id != pending_snapshot.message_id
    {
        log_api_result(
            "answer unauthorized callback query",
            state
                .api
                .answer_callback(&callback.id, Some("此按钮不属于你的操作。")),
        );
        return Err(TelegramError::UnauthorizedCallback);
    }
    log_api_result(
        "answer callback query",
        state.api.answer_callback(&callback.id, None),
    );

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
        let collect_result = match state.database.lock() {
            Ok(mut database) => database.collect_image(&pending.source_path),
            Err(_) => Err(memelith_core::Error::InvalidDatabase(
                "database lock was poisoned".to_owned(),
            )),
        };
        if let Err(error) = collect_result {
            let pending_id = pending.id;
            state
                .pending
                .lock()
                .map_err(|_| TelegramError::Api("pending operation lock was poisoned".to_owned()))?
                .insert(pending_id, pending.clone());
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
    delete_message_or_queue(&state, pending.chat_id, pending.message_id)?;
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

fn delete_message_or_queue(
    state: &BotState,
    chat_id: i64,
    message_id: i64,
) -> Result<bool, TelegramError> {
    match state.api.delete_message(chat_id, message_id) {
        Ok(true) => Ok(true),
        Ok(false) => {
            queue_message_deletion(state, chat_id, message_id)?;
            eprintln!("Memelith Telegram deleteMessage returned false for {chat_id}:{message_id}");
            Ok(false)
        }
        Err(error) => {
            queue_message_deletion(state, chat_id, message_id)?;
            eprintln!("Memelith Telegram failed to delete {chat_id}:{message_id}: {error}");
            Ok(false)
        }
    }
}

fn queue_message_deletion(
    state: &BotState,
    chat_id: i64,
    message_id: i64,
) -> Result<(), TelegramError> {
    state
        .pending_deletions
        .lock()
        .map_err(|_| TelegramError::Api("deletion queue lock was poisoned".to_owned()))?
        .insert((chat_id, message_id));
    Ok(())
}

fn retry_pending_deletions(state: &BotState) {
    let pending = match state.pending_deletions.lock() {
        Ok(mut pending) => std::mem::take(&mut *pending),
        Err(_) => {
            eprintln!("Memelith Telegram failed to acquire deletion queue lock");
            return;
        }
    };
    for (chat_id, message_id) in pending {
        match state.api.delete_message(chat_id, message_id) {
            Ok(true) => {}
            Ok(false) => {
                if let Err(error) = queue_message_deletion(state, chat_id, message_id) {
                    eprintln!("Memelith Telegram failed to requeue deletion: {error}");
                }
            }
            Err(error) => {
                eprintln!(
                    "Memelith Telegram deletion retry failed for {chat_id}:{message_id}: {error}"
                );
                if let Err(queue_error) = queue_message_deletion(state, chat_id, message_id) {
                    eprintln!("Memelith Telegram failed to requeue deletion: {queue_error}");
                }
            }
        }
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

struct TempFileGuard {
    path: PathBuf,
    armed: bool,
}

impl TempFileGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if self.armed {
            remove_temp_file(&self.path);
        }
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
    from: Option<User>,
    #[serde(default)]
    photo: Vec<PhotoSize>,
}

#[derive(Clone, Debug, Deserialize)]
struct User {
    id: i64,
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
    from: User,
    message: Option<Message>,
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
