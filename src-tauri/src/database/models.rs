
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Type)]
pub struct AssetText {
    pub id: String,
    pub text: String,
    pub create_at: NaiveDateTime,
    pub sha256: Option<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Type)]
pub struct AssetImage {
    pub id: String,
    pub image: String,
    pub create_at: NaiveDateTime,
    pub sha256: Option<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Type)]
pub struct AssetVideo {
    pub id: String,
    pub video: String,
    pub create_at: NaiveDateTime,
    pub sha256: Option<String>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, Type)]
#[serde(tag = "type")]
pub enum Asset{
    #[serde(rename = "text")]
    Text(AssetText),
    #[serde(rename = "image")]
    Image(AssetImage),
    #[serde(rename = "video")]
    Video(AssetVideo),
}

impl From<AssetText> for Asset{
    fn from(asset: AssetText)->Self{
        Asset::Text(asset)
    }
}
impl From<AssetImage> for Asset{
    fn from(asset: AssetImage)->Self{
        Asset::Image(asset)
    }
}
impl From<AssetVideo> for Asset{
    fn from(asset: AssetVideo)->Self{
        Asset::Video(asset)
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Type)]
pub struct Sticker{
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub create_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
    pub collection: String,
    pub tags: Vec<String>,
    pub assets: Vec<Asset>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Type)]
pub struct Collection{
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub create_at: NaiveDateTime,
    pub modified_at: NaiveDateTime,
    pub tags: Vec<String>,
    pub preview: Option<Asset>,
}
