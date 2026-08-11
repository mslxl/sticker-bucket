use std::path::PathBuf;

use uuid::Uuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemePack {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Meme {
    pub id: Uuid,
    pub meme_pack_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub contents: Vec<MemeContent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemeContent {
    Image(MemeImage),
    Text(MemeText),
}

impl MemeContent {
    pub fn id(&self) -> Uuid {
        match self {
            Self::Image(image) => image.id,
            Self::Text(text) => text.id,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemeImage {
    pub id: Uuid,
    pub relative_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub byte_size: u64,
    pub format: ImageFormat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimilarMemeImage {
    pub content_id: Uuid,
    pub meme_id: Uuid,
    pub meme_name: Option<String>,
    pub relative_path: PathBuf,
    pub cosine_distance: f32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemeText {
    pub id: Uuid,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectorItem {
    pub id: Uuid,
    pub content: CollectorContent,
    pub duplicate: Option<CollectorDuplicate>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CollectorContent {
    Image {
        relative_path: PathBuf,
        width: u32,
        height: u32,
        byte_size: u64,
        format: ImageFormat,
    },
    Text {
        text: String,
    },
}

/// Current duplicate status for an item in the Collector.
///
/// Similarity status can be dismissed explicitly. Statuses are rechecked when
/// the library changes so a deleted duplicate target does not strand an item.
#[derive(Clone, Debug, PartialEq)]
pub enum CollectorDuplicate {
    Hash {
        target: CollectorDuplicateTarget,
    },
    Similarity {
        target: CollectorDuplicateTarget,
        cosine_distance: f32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectorDuplicateSource {
    Collector,
    Meme,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollectorDuplicateTarget {
    /// The target's current location. Promoting a Collector target updates
    /// this to `Meme` while preserving `content_id`.
    pub source: CollectorDuplicateSource,
    /// The content identifier used to revalidate this duplicate relationship.
    pub content_id: Uuid,
    /// Present only while a Meme target still exists.
    pub meme_id: Option<Uuid>,
    /// Present only while a named Meme target still exists.
    pub meme_name: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
    Gif,
}

impl ImageFormat {
    pub(crate) const fn as_database_str(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::WebP => "webp",
            Self::Gif => "gif",
        }
    }

    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Gif => "gif",
        }
    }

    pub(crate) fn from_database_str(value: &str) -> Option<Self> {
        match value {
            "png" => Some(Self::Png),
            "jpeg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "gif" => Some(Self::Gif),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectiveTag {
    pub tag: Tag,
    pub direct: bool,
    pub inherited: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewMemePack {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateMemePack {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewMeme {
    pub name: Option<String>,
    pub description: Option<String>,
    pub contents: Vec<NewMemeContent>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewMemeFromCollector {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NewMemeContent {
    Image { source_path: PathBuf },
    Text { text: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateMemeMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NewTag {
    pub name: String,
}
