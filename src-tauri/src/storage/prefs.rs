use serde::{Deserialize, Serialize};
use specta::Type;

use super::db::Tag;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct StoragePrefs {
    // readonly_mode: bool,

    // password_salt_hash: Option<String>,
    nsfw_mode: bool,
    lock_libary_when_opened: bool,
    disable_nsfw_mode_on_startup: bool,
    disable_nsfw_mode_on_ip_changed: bool,
    hide_nsfw_mode_option: bool,
    flatten_assets: bool,
    keep_extension: bool,
    // share_nsfw: bool,
    // nsfw_tags: Vec<Tag>,
    // advanced_search: bool,

    // use_thumbnail: bool,
    // use_thumbnail_threshold: Option<usize>,
    // prefer_static_thumbnail: bool,
    // prefer_movie_thumbnail: bool,

    // auto_clean: bool,
    // auto_maintain: bool,
    // auto_backup_metadata: Option<u128>,
    // auto_backup_library: Option<u128>,

    // check_password_on_ip_changed: bool,
    // check_password_after_second: Option<u128>,
    // check_password_on_startup: bool,
    // check_password_on_nsfw_option_changed: bool,

    // last_ip_hash: u64,
    // last_startup: u128
}
