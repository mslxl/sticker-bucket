use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Serialize, Deserialize, Clone, Type)]
pub struct Tag {
    namespace: String,
    value: String,
}
