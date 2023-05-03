use std::path::PathBuf;

use crate::{error::Error, meme::interfer};

use serde::Serialize;
use tauri::api::dialog;

use super::database::Tag;

#[derive(Serialize)]
pub struct InterferedMeme {
    path: PathBuf,
    summary: Option<String>,
    tags: Vec<Tag>,
}

#[tauri::command]
pub async fn open_image_and_interfere() -> Result<Option<InterferedMeme>, Error> {
    let path = tauri::api::dialog::blocking::FileDialogBuilder::new()
        .add_filter("Image", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
        .set_title("Add")
        .pick_file();

    if let Some(path) = path {
        interfere_image(path).await
    } else {
        Err(Error::UserCancel)
    }
}

#[tauri::command]
pub async fn interfere_image(path: PathBuf) -> Result<Option<InterferedMeme>, Error> {
    let summary = interfer::interfer_summary(&path);
    let tags = interfer::interfer_tags(&path, &summary);

    Ok(Some(InterferedMeme {
        path,
        summary,
        tags,
    }))
}

#[tauri::command]
pub async fn open_picture_list() -> Result<Vec<String>, Error> {
    let (tx, rx) = std::sync::mpsc::channel();
    let dialog = dialog::FileDialogBuilder::new();
    dialog.pick_folder(move |f| {
        tx.send(f).unwrap();
    });
    if let Some(dir) = rx.recv()? {
        let files = std::fs::read_dir(dir)?
            .into_iter()
            .filter(|p| {
                if let Ok(f) = p {
                    f.file_type().unwrap().is_file()
                } else {
                    false
                }
            })
            .map(|v| v.unwrap().path())
            .filter(|f| {
                f.extension().is_some()
                    && match f.extension().unwrap().to_string_lossy().as_ref() {
                        "jpeg" | "jpg" | "png" | "webp" | "gif" | "bmp" => true,
                        _ => false,
                    }
            })
            .map(|p| p.canonicalize().unwrap().to_str().unwrap().to_owned())
            .collect();
        Ok(files)
    } else {
        Err(Error::UserCancel)
    }
}
