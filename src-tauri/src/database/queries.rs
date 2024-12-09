use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::database::models::{Asset, AssetImage, AssetText, AssetVideo, Collection, Sticker};


pub fn insert_asset(conn: &Connection, asset: &Asset)->anyhow::Result<Asset>{
    // First check if the asset already exists
    let (typ, ref_content) = match asset {
        Asset::Text(text_asset) => ("text", &text_asset.text),
        Asset::Image(image_asset) => ("image", &image_asset.image),
        Asset::Video(video_asset) => ("video", &video_asset.video),
    };

    // Try to find existing asset
    let existing_asset: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT id FROM asset WHERE typ = ?1 AND ref = ?2",
        (typ, ref_content),
        |row| row.get(0)
    );

    if let Ok(existing_id) = existing_asset {
        // Asset already exists, construct and return it
        let asset_data: (String, String, String, chrono::NaiveDateTime, Option<String>) = conn.query_row(
            "SELECT id, typ, ref, create_at, sha256 FROM asset WHERE id = ?1",
            [existing_id],
            |row| Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?
            ))
        )?;

        return match asset_data.1.as_str() {
            "text" => Ok(Asset::Text(crate::database::models::AssetText {
                id: asset_data.0,
                text: asset_data.2,
                create_at: asset_data.3,
                sha256: asset_data.4,
            })),
            "image" => Ok(Asset::Image(crate::database::models::AssetImage {
                id: asset_data.0,
                image: asset_data.2,
                create_at: asset_data.3,
                sha256: asset_data.4,
            })),
            "video" => Ok(Asset::Video(crate::database::models::AssetVideo {
                id: asset_data.0,
                video: asset_data.2,
                create_at: asset_data.3,
                sha256: asset_data.4,
            })),
            _ => Err(anyhow::anyhow!("Invalid asset type in database"))
        };
    }

    // If no existing asset found, proceed with insertion
    match asset {
        Asset::Text(text_asset) => {
            conn.execute(
                "INSERT INTO asset (id, typ, ref, create_at, sha256) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &text_asset.id,
                    "text",
                    &text_asset.text,
                    &text_asset.create_at,
                    &text_asset.sha256
                )
            )?;
            Ok(asset.clone())
        },
        Asset::Image(image_asset) => {
            conn.execute(
                "INSERT INTO asset (id, typ, ref, create_at, sha256) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &image_asset.id,
                    "image",
                    &image_asset.image,
                    &image_asset.create_at,
                    &image_asset.sha256
                )
            )?;
            Ok(asset.clone())
        },
        Asset::Video(video_asset) => {
            conn.execute(
                "INSERT INTO asset (id, typ, ref, create_at, sha256) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    &video_asset.id,
                    "video",
                    &video_asset.video,
                    &video_asset.create_at,
                    &video_asset.sha256
                )
            )?;
            Ok(asset.clone())
        }
    }
}

pub struct InsertStickerQuery {
    pub name: Option<String>,
    pub description: Option<String>,
    pub collection_id: CollectionId,
    pub tags: Vec<String>,
    pub assets: Vec<Asset>,
}

pub fn insert_sticker(conn: &Connection, sticker: &InsertStickerQuery)->anyhow::Result<()>{
    let sticker_id = Uuid::new_v4().to_string();
    // Insert the sticker
    conn.execute(
        "INSERT INTO stickers (id, name, description, collection_id) 
         VALUES (?1, ?2, ?3, ?4)",
        (
            &sticker_id,
            &sticker.name,
            &sticker.description,
            &sticker.collection_id
        )
    )?;

    // Handle assets
    for asset in &sticker.assets {
        // First insert the asset if it doesn't exist
        insert_asset(conn, asset)?;

        // Then create the asset-sticker relationship
        conn.execute(
            "INSERT OR IGNORE INTO asset_stickers (asset_id, sticker_id) VALUES (?1, ?2)",
            (
                match asset {
                    Asset::Text(a) => &a.id,
                    Asset::Image(a) => &a.id,
                    Asset::Video(a) => &a.id,
                },
                &sticker_id
            )
        )?;
    }

    // Handle tags
    for tag_name in &sticker.tags {
        // First get or create the tag
        let tag_id: i64 = conn.query_row(
            "INSERT OR IGNORE INTO tag (name) VALUES (?1) RETURNING id",
            [tag_name],
            |row| row.get(0)
        ).or_else(|_| {
            // If the tag already exists, get its id
            conn.query_row(
                "SELECT id FROM tag WHERE name = ?1",
                [tag_name],
                |row| row.get(0)
            )
        })?;

        // Create the tag-sticker relationship
        conn.execute(
            "INSERT OR IGNORE INTO tag_stickers (tag_id, sticker_id) VALUES (?1, ?2)",
            (tag_id, &sticker_id)
        )?;
    }

    Ok(())
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Type)]
pub struct InsertCollectionQuery {
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
}


pub type CollectionId = String;
pub fn insert_collection(conn: &Connection, collection: &InsertCollectionQuery)->anyhow::Result<CollectionId>{
    let collection_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO collections (id, name, description, author) VALUES (?1, ?2, ?3, ?4)",
        (
            &collection_id,
            &collection.name,
            &collection.description,
            &collection.author,
        )
    )?;
    Ok(collection_id)
}

pub fn search_collection(conn: &Connection, name: &str)->anyhow::Result<Vec<Collection>>{
    let mut stmt = conn.prepare(
        "SELECT id, name, description, author, create_at, modified_at 
         FROM collections 
         WHERE name LIKE ?1"
    )?;

    let rows = stmt.query_map([format!("%{}%", name)], |row| {
        get_collection_from_row(conn, row)
    })?;
    
    let mut collections = Vec::new();
    for row in rows {
        collections.push(row?);
    }

    Ok(collections)
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Type)]
pub enum GetCollectionsOrder{
    ASC,
    DESC,
}

/// Converts a database row into a Collection struct with all related data.
/// 
/// This function performs several database operations to construct a complete Collection:
/// 1. Extracts basic collection information from the provided row
/// 2. Fetches all tags associated with the collection
/// 3. Retrieves a preview asset from the most recently modified sticker in the collection
/// 
/// # Arguments
/// * `conn` - Database connection reference
/// * `row` - Database row containing collection data (id, name, description, author, create_at, modified_at)
/// 
/// # Returns
/// * `rusqlite::Result<Collection>` - The constructed Collection struct or an error
/// 
/// # Database Operations
/// - Queries tag_collections and tag tables to get collection tags
/// - Queries stickers, asset_stickers, and asset tables to get preview asset
/// - Uses LEFT JOIN to handle cases where collections have no stickers/assets
/// 
/// # Performance Considerations
/// - This function performs multiple database queries per collection
/// - Consider caching results if called frequently for the same collections
/// - The preview asset query uses ORDER BY and LIMIT for efficiency
/// 
/// # Error Handling
/// - Returns rusqlite::Error for database operation failures
/// - Gracefully handles collections with no tags or preview assets
/// - Invalid asset types in the database will cause errors
/// 
/// # Example Usage
/// ```rust
/// let collection = get_collection_from_row(&conn, &row)?;
/// println!("Collection: {} with {} tags", collection.name, collection.tags.len());
/// ```
fn get_collection_from_row(conn: &Connection, row: &rusqlite::Row) -> rusqlite::Result<Collection> {
    let id: String = row.get(0)?;
    
    // Get tags for this collection
    let mut tag_stmt = conn.prepare(
        "SELECT t.name 
         FROM tag t
         JOIN tag_collections tc ON t.id = tc.tag_id
         WHERE tc.collection_id = ?1"
    )?;
    
    let tag_rows = tag_stmt.query_map([&id], |row| {
        Ok(row.get::<_, String>(0)?)
    })?;
    
    let tags: Vec<String> = tag_rows.collect::<Result<Vec<_>, _>>()?;

    // Get preview from latest sticker's first asset
    // Updated query to include sha256 field from the asset table
    let preview: Option<Asset> = conn.query_row(
        "SELECT a.id, a.typ, a.ref, a.create_at, a.sha256
         FROM stickers s
         JOIN asset_stickers ast ON s.id = ast.sticker_id
         JOIN asset a ON ast.asset_id = a.id
         WHERE s.collection_id = ?1
         ORDER BY s.modified_at DESC
         LIMIT 1",
        [&id],
        |row| {
            let typ: String = row.get(1)?;
            let asset = match typ.as_str() {
                "text" => Asset::Text(AssetText {
                    id: row.get(0)?,
                    text: row.get(2)?,
                    create_at: row.get(3)?,
                    sha256: row.get(4)?, // sha256 field from database
                }),
                "image" => Asset::Image(AssetImage {
                    id: row.get(0)?,
                    image: row.get(2)?,
                    create_at: row.get(3)?,
                    sha256: row.get(4)?, // sha256 field from database
                }),
                "video" => Asset::Video(AssetVideo {
                    id: row.get(0)?,
                    video: row.get(2)?,
                    create_at: row.get(3)?,
                    sha256: row.get(4)?, // sha256 field from database
                }),
                _ => return Err(rusqlite::Error::InvalidParameterName("Invalid asset type".into())),
            };
            Ok(Some(asset))
        }
    ).unwrap_or(None);
        
    Ok(Collection{
        id,
        name: row.get(1)?,
        description: row.get(2)?,
        author: row.get(3)?,
        create_at: row.get(4)?,
        modified_at: row.get(5)?,
        tags,
        preview,
    })
}

pub fn get_collections(
    conn: &Connection,
    order: GetCollectionsOrder,
    cursor: Option<String>,
    limit: u32,
)->anyhow::Result<Vec<Collection>>{
    let order_clause = match order {
        GetCollectionsOrder::ASC => "ASC",
        GetCollectionsOrder::DESC => "DESC",
    };

    let cursor_condition = match cursor {
        Some(cursor_id) => {
            match order {
                GetCollectionsOrder::ASC => format!("AND id > '{}'", cursor_id),
                GetCollectionsOrder::DESC => format!("AND id < '{}'", cursor_id),
            }
        },
        None => String::new(),
    };

    let mut stmt = conn.prepare(&format!(
        "SELECT id, name, description, author, create_at, modified_at 
         FROM collections 
         WHERE 1=1 {}
         ORDER BY id {}
         LIMIT ?1",
        cursor_condition,
        order_clause
    ))?;

    let rows = stmt.query_map([limit], |row| {
        get_collection_from_row(conn, row)
    })?;

    let mut collections = Vec::new();
    for row in rows {
        collections.push(row?);
    }

    Ok(collections)
}

pub mod tests{
    use crate::database::models::AssetText;

    use super::*;

    #[test]
    fn test_asset_text(){
        let asset = Asset::Text(AssetText{
            id: "1".to_string(),
            text: "Hello, world!".to_string(),
            create_at: chrono::Local::now().naive_local(),
            sha256: None,
        });
        let serialized = serde_json::to_string(&asset).unwrap();
        println!("{}", serialized);

        let deserialized: Asset = serde_json::from_str(&serialized).unwrap();
        assert_eq!(asset, deserialized);
    }
}