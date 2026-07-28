use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io,
    path::{Component, Path, PathBuf},
    time::Duration,
};

use image::{DynamicImage, ImageFormat as DecodedImageFormat, ImageReader};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::{
    EffectiveTag, EmbeddingProvider, Error, ImageFormat, Meme, MemeContent, MemeImage, MemePack,
    MemeText, NewMeme, NewMemeContent, NewMemePack, NewTag, Result, Tag, UpdateMemeMetadata,
    UpdateMemePack,
};

const SCHEMA_VERSION: i64 = 1;
const DATABASE_FILENAME: &str = "memelith.sqlite3";
const MEDIA_DIRECTORY: &str = "media/images";
const STAGING_DIRECTORY: &str = ".staging";

pub struct MemeDatabase {
    storage_root: PathBuf,
    database_path: PathBuf,
    connection: Connection,
    embedding_provider: Box<dyn EmbeddingProvider>,
    embedding_dimension: usize,
}

impl std::fmt::Debug for MemeDatabase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MemeDatabase")
            .field("storage_root", &self.storage_root)
            .field("database_path", &self.database_path)
            .field("embedding_model", &self.embedding_provider.model_id())
            .field("embedding_dimension", &self.embedding_dimension)
            .finish_non_exhaustive()
    }
}

impl MemeDatabase {
    pub fn open(
        storage_root: impl AsRef<Path>,
        embedding_provider: impl EmbeddingProvider + 'static,
    ) -> Result<Self> {
        let model_id = embedding_provider.model_id().trim();
        if model_id.is_empty() {
            return Err(Error::EmptyEmbeddingModelId);
        }
        let embedding_dimension = embedding_provider.dimension();
        if embedding_dimension == 0 {
            return Err(Error::EmptyEmbeddingDimension);
        }

        let requested_root = storage_root.as_ref();
        if requested_root.exists() && !requested_root.is_dir() {
            return Err(Error::InvalidStorageRoot(requested_root.to_path_buf()));
        }
        fs::create_dir_all(requested_root)?;
        let storage_root = fs::canonicalize(requested_root)?;
        fs::create_dir_all(storage_root.join(MEDIA_DIRECTORY))?;
        fs::create_dir_all(storage_root.join(STAGING_DIRECTORY))?;

        let database_path = storage_root.join(DATABASE_FILENAME);
        let mut connection = Connection::open(&database_path)?;
        configure_connection(&connection)?;
        migrate_database(&mut connection, model_id, embedding_dimension)?;

        let database = Self {
            storage_root,
            database_path,
            connection,
            embedding_provider: Box::new(embedding_provider),
            embedding_dimension,
        };
        database.clean_staging_directory()?;
        database.collect_orphaned_media()?;
        Ok(database)
    }

    pub fn storage_root(&self) -> &Path {
        &self.storage_root
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn resolve_media_path(&self, relative_path: impl AsRef<Path>) -> Result<PathBuf> {
        resolve_media_path(&self.storage_root, relative_path.as_ref())
    }

    pub fn create_meme_pack(&mut self, input: NewMemePack) -> Result<MemePack> {
        let input = validate_meme_pack_input(input)?;
        let name_embedding = self.embed_text("MemePack name", &input.name)?;
        let description_embedding =
            self.embed_optional_text("MemePack description", input.description.as_deref())?;
        let id = Uuid::new_v4();

        self.connection.execute(
            "INSERT INTO meme_packs(
                id, name, name_embedding, description, description_embedding, author, source
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id.to_string(),
                input.name,
                name_embedding,
                input.description,
                description_embedding,
                input.author,
                input.source,
            ],
        )?;

        self.get_meme_pack(id)
    }

    pub fn get_meme_pack(&self, id: Uuid) -> Result<MemePack> {
        self.connection
            .query_row(
                "SELECT id, name, description, author, source FROM meme_packs WHERE id = ?1",
                [id.to_string()],
                map_meme_pack,
            )
            .optional()?
            .ok_or(Error::MemePackNotFound(id))
    }

    pub fn list_meme_packs(&self) -> Result<Vec<MemePack>> {
        let mut statement = self.connection.prepare(
            "SELECT id, name, description, author, source
             FROM meme_packs
             ORDER BY name COLLATE NOCASE, id",
        )?;
        statement
            .query_map([], map_meme_pack)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    pub fn update_meme_pack(&mut self, id: Uuid, input: UpdateMemePack) -> Result<MemePack> {
        self.ensure_meme_pack(id)?;
        let input = validate_update_meme_pack_input(input)?;
        let name_embedding = self.embed_text("MemePack name", &input.name)?;
        let description_embedding =
            self.embed_optional_text("MemePack description", input.description.as_deref())?;
        self.connection.execute(
            "UPDATE meme_packs
             SET name = ?2, name_embedding = ?3, description = ?4,
                 description_embedding = ?5, author = ?6, source = ?7
             WHERE id = ?1",
            params![
                id.to_string(),
                input.name,
                name_embedding,
                input.description,
                description_embedding,
                input.author,
                input.source,
            ],
        )?;
        self.get_meme_pack(id)
    }

    pub fn delete_meme_pack(&mut self, id: Uuid) -> Result<()> {
        self.ensure_meme_pack(id)?;
        let media = self.image_paths_for_meme_pack(id)?;
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM meme_packs WHERE id = ?1", [id.to_string()])?;
        transaction.commit()?;
        remove_managed_files(&media)
    }

    pub fn create_meme(&mut self, meme_pack_id: Uuid, input: NewMeme) -> Result<Meme> {
        self.ensure_meme_pack(meme_pack_id)?;
        let (name, description) = validate_meme_metadata(input.name, input.description)?;
        if input.contents.is_empty() {
            return Err(Error::EmptyMemeContents);
        }
        let name_embedding = self.embed_optional_text("Meme name", name.as_deref())?;
        let description_embedding =
            self.embed_optional_text("Meme description", description.as_deref())?;
        let contents = self.prepare_contents(input.contents)?;
        let mut pending_files =
            PendingFiles::stage(&self.storage_root.join(STAGING_DIRECTORY), &contents)?;
        let id = Uuid::new_v4();

        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO memes(
                id, meme_pack_id, name, name_embedding, description, description_embedding
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                id.to_string(),
                meme_pack_id.to_string(),
                name,
                name_embedding,
                description,
                description_embedding,
            ],
        )?;
        insert_contents(&transaction, id, &contents)?;
        pending_files.promote()?;
        transaction.commit()?;
        pending_files.disarm();
        self.get_meme(id)
    }

    pub fn get_meme(&self, id: Uuid) -> Result<Meme> {
        let raw = self
            .connection
            .query_row(
                "SELECT id, meme_pack_id, name, description FROM memes WHERE id = ?1",
                [id.to_string()],
                |row| {
                    Ok((
                        uuid_from_column(row, 0)?,
                        uuid_from_column(row, 1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .optional()?
            .ok_or(Error::MemeNotFound(id))?;
        let contents = self.contents_for_meme(id)?;
        if contents.is_empty() {
            return Err(Error::InvalidDatabase(format!(
                "Meme {id} has no content blocks"
            )));
        }
        Ok(Meme {
            id: raw.0,
            meme_pack_id: raw.1,
            name: raw.2,
            description: raw.3,
            contents,
        })
    }

    pub fn list_memes(&self, meme_pack_id: Uuid) -> Result<Vec<Meme>> {
        self.ensure_meme_pack(meme_pack_id)?;
        let ids = {
            let mut statement = self
                .connection
                .prepare("SELECT id FROM memes WHERE meme_pack_id = ?1 ORDER BY rowid")?;
            statement
                .query_map([meme_pack_id.to_string()], |row| uuid_from_column(row, 0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        ids.into_iter().map(|id| self.get_meme(id)).collect()
    }

    /// Lists every Meme independently of its MemePack, newest first.
    pub fn list_all_memes(&self) -> Result<Vec<Meme>> {
        let ids = {
            let mut statement = self
                .connection
                .prepare("SELECT id FROM memes ORDER BY rowid DESC")?;
            statement
                .query_map([], |row| uuid_from_column(row, 0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        ids.into_iter().map(|id| self.get_meme(id)).collect()
    }

    pub fn update_meme_metadata(&mut self, id: Uuid, input: UpdateMemeMetadata) -> Result<Meme> {
        self.ensure_meme(id)?;
        let (name, description) = validate_meme_metadata(input.name, input.description)?;
        let name_embedding = self.embed_optional_text("Meme name", name.as_deref())?;
        let description_embedding =
            self.embed_optional_text("Meme description", description.as_deref())?;
        self.connection.execute(
            "UPDATE memes
             SET name = ?2, name_embedding = ?3,
                 description = ?4, description_embedding = ?5
             WHERE id = ?1",
            params![
                id.to_string(),
                name,
                name_embedding,
                description,
                description_embedding,
            ],
        )?;
        self.get_meme(id)
    }

    pub fn move_meme(&mut self, id: Uuid, destination_meme_pack_id: Uuid) -> Result<Meme> {
        self.ensure_meme(id)?;
        self.ensure_meme_pack(destination_meme_pack_id)?;
        self.connection.execute(
            "UPDATE memes SET meme_pack_id = ?2 WHERE id = ?1",
            params![id.to_string(), destination_meme_pack_id.to_string()],
        )?;
        self.get_meme(id)
    }

    pub fn replace_meme_contents(
        &mut self,
        id: Uuid,
        contents: Vec<NewMemeContent>,
    ) -> Result<Meme> {
        self.ensure_meme(id)?;
        if contents.is_empty() {
            return Err(Error::EmptyMemeContents);
        }
        let old_media = self.image_paths_for_meme(id)?;
        let contents = self.prepare_contents(contents)?;
        let mut pending_files =
            PendingFiles::stage(&self.storage_root.join(STAGING_DIRECTORY), &contents)?;

        let transaction = self.connection.transaction()?;
        transaction.execute(
            "DELETE FROM meme_contents WHERE meme_id = ?1",
            [id.to_string()],
        )?;
        insert_contents(&transaction, id, &contents)?;
        pending_files.promote()?;
        transaction.commit()?;
        pending_files.disarm();
        remove_managed_files(&old_media)?;
        self.get_meme(id)
    }

    pub fn delete_meme(&mut self, id: Uuid) -> Result<()> {
        self.ensure_meme(id)?;
        let media = self.image_paths_for_meme(id)?;
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM memes WHERE id = ?1", [id.to_string()])?;
        transaction.commit()?;
        remove_managed_files(&media)
    }

    pub fn create_tag(&mut self, input: NewTag) -> Result<Tag> {
        let name = required_text(input.name, "Tag name")?;
        let normalized_name = normalize_tag_name(&name);
        if self.tag_name_exists(&normalized_name, None)? {
            return Err(Error::DuplicateTag(name));
        }
        let name_embedding = self.embed_text("Tag name", &name)?;
        let id = Uuid::new_v4();
        self.connection.execute(
            "INSERT INTO tags(id, name, normalized_name, name_embedding)
             VALUES (?1, ?2, ?3, ?4)",
            params![id.to_string(), name, normalized_name, name_embedding],
        )?;
        self.get_tag(id)
    }

    pub fn get_tag(&self, id: Uuid) -> Result<Tag> {
        self.connection
            .query_row(
                "SELECT id, name FROM tags WHERE id = ?1",
                [id.to_string()],
                map_tag,
            )
            .optional()?
            .ok_or(Error::TagNotFound(id))
    }

    pub fn list_tags(&self) -> Result<Vec<Tag>> {
        let mut statement = self
            .connection
            .prepare("SELECT id, name FROM tags ORDER BY normalized_name, id")?;
        statement
            .query_map([], map_tag)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    pub fn rename_tag(&mut self, id: Uuid, name: String) -> Result<Tag> {
        self.ensure_tag(id)?;
        let name = required_text(name, "Tag name")?;
        let normalized_name = normalize_tag_name(&name);
        if self.tag_name_exists(&normalized_name, Some(id))? {
            return Err(Error::DuplicateTag(name));
        }
        let name_embedding = self.embed_text("Tag name", &name)?;
        self.connection.execute(
            "UPDATE tags
             SET name = ?2, normalized_name = ?3, name_embedding = ?4
             WHERE id = ?1",
            params![id.to_string(), name, normalized_name, name_embedding],
        )?;
        self.get_tag(id)
    }

    pub fn delete_tag(&mut self, id: Uuid) -> Result<()> {
        self.ensure_tag(id)?;
        self.connection
            .execute("DELETE FROM tags WHERE id = ?1", [id.to_string()])?;
        Ok(())
    }

    pub fn attach_tag_to_meme_pack(&mut self, meme_pack_id: Uuid, tag_id: Uuid) -> Result<()> {
        self.ensure_meme_pack(meme_pack_id)?;
        self.ensure_tag(tag_id)?;
        insert_tag_association(
            &self.connection,
            "meme_pack_tags",
            "meme_pack_id",
            "MemePack",
            meme_pack_id,
            tag_id,
        )
    }

    pub fn detach_tag_from_meme_pack(&mut self, meme_pack_id: Uuid, tag_id: Uuid) -> Result<()> {
        self.ensure_meme_pack(meme_pack_id)?;
        self.ensure_tag(tag_id)?;
        delete_tag_association(
            &self.connection,
            "meme_pack_tags",
            "meme_pack_id",
            "MemePack",
            meme_pack_id,
            tag_id,
        )
    }

    pub fn attach_tag_to_meme(&mut self, meme_id: Uuid, tag_id: Uuid) -> Result<()> {
        self.ensure_meme(meme_id)?;
        self.ensure_tag(tag_id)?;
        insert_tag_association(
            &self.connection,
            "meme_tags",
            "meme_id",
            "Meme",
            meme_id,
            tag_id,
        )
    }

    pub fn detach_tag_from_meme(&mut self, meme_id: Uuid, tag_id: Uuid) -> Result<()> {
        self.ensure_meme(meme_id)?;
        self.ensure_tag(tag_id)?;
        delete_tag_association(
            &self.connection,
            "meme_tags",
            "meme_id",
            "Meme",
            meme_id,
            tag_id,
        )
    }

    pub fn list_meme_pack_tags(&self, meme_pack_id: Uuid) -> Result<Vec<Tag>> {
        self.ensure_meme_pack(meme_pack_id)?;
        let mut statement = self.connection.prepare(
            "SELECT t.id, t.name
             FROM tags t
             JOIN meme_pack_tags pt ON pt.tag_id = t.id
             WHERE pt.meme_pack_id = ?1
             ORDER BY t.normalized_name, t.id",
        )?;
        statement
            .query_map([meme_pack_id.to_string()], map_tag)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    pub fn list_meme_direct_tags(&self, meme_id: Uuid) -> Result<Vec<Tag>> {
        self.ensure_meme(meme_id)?;
        let mut statement = self.connection.prepare(
            "SELECT t.id, t.name
             FROM tags t
             JOIN meme_tags mt ON mt.tag_id = t.id
             WHERE mt.meme_id = ?1
             ORDER BY t.normalized_name, t.id",
        )?;
        statement
            .query_map([meme_id.to_string()], map_tag)?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    pub fn list_meme_effective_tags(&self, meme_id: Uuid) -> Result<Vec<EffectiveTag>> {
        self.ensure_meme(meme_id)?;
        let mut statement = self.connection.prepare(
            "SELECT
                 t.id,
                 t.name,
                 EXISTS(
                     SELECT 1 FROM meme_tags mt
                     WHERE mt.meme_id = ?1 AND mt.tag_id = t.id
                 ) AS direct,
                 EXISTS(
                     SELECT 1
                     FROM memes m
                     JOIN meme_pack_tags pt ON pt.meme_pack_id = m.meme_pack_id
                     WHERE m.id = ?1 AND pt.tag_id = t.id
                 ) AS inherited
             FROM tags t
             WHERE direct OR inherited
             ORDER BY t.normalized_name, t.id",
        )?;
        statement
            .query_map([meme_id.to_string()], |row| {
                Ok(EffectiveTag {
                    tag: Tag {
                        id: uuid_from_column(row, 0)?,
                        name: row.get(1)?,
                    },
                    direct: row.get(2)?,
                    inherited: row.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(Error::from)
    }

    fn embed_text(&mut self, field: &'static str, text: &str) -> Result<Vec<u8>> {
        let vector = self
            .embedding_provider
            .embed_text(text)
            .map_err(Error::EmbeddingProvider)?;
        encode_embedding(field, vector, self.embedding_dimension)
    }

    fn embed_optional_text(
        &mut self,
        field: &'static str,
        text: Option<&str>,
    ) -> Result<Option<Vec<u8>>> {
        text.map(|text| self.embed_text(field, text)).transpose()
    }

    fn prepare_contents(&mut self, contents: Vec<NewMemeContent>) -> Result<Vec<PreparedContent>> {
        contents
            .into_iter()
            .map(|content| self.prepare_content(content))
            .collect()
    }

    fn prepare_content(&mut self, content: NewMemeContent) -> Result<PreparedContent> {
        let id = Uuid::new_v4();
        match content {
            NewMemeContent::Text { text } => {
                let text = required_text(text, "Meme text")?;
                let embedding = self.embed_text("Meme text", &text)?;
                Ok(PreparedContent::Text {
                    id,
                    text,
                    embedding,
                })
            }
            NewMemeContent::Image { source_path } => {
                let (format, decoded) = decode_supported_image(&source_path)?;
                let metadata = fs::metadata(&source_path)?;
                if !metadata.is_file() {
                    return Err(Error::UnsupportedImageFormat(source_path));
                }
                let width = decoded.width();
                let height = decoded.height();
                let vector = self
                    .embedding_provider
                    .embed_image(&decoded)
                    .map_err(Error::EmbeddingProvider)?;
                let embedding = encode_embedding("Meme image", vector, self.embedding_dimension)?;
                let relative_path =
                    PathBuf::from(format!("{MEDIA_DIRECTORY}/{id}.{}", format.extension()));
                let final_path = resolve_media_path(&self.storage_root, &relative_path)?;
                Ok(PreparedContent::Image {
                    id,
                    source_path,
                    relative_path,
                    final_path,
                    width,
                    height,
                    byte_size: metadata.len(),
                    format,
                    embedding,
                })
            }
        }
    }

    fn contents_for_meme(&self, meme_id: Uuid) -> Result<Vec<MemeContent>> {
        let rows = {
            let mut statement = self.connection.prepare(
                "SELECT id, kind, text, relative_path, width, height, byte_size, image_format
                 FROM meme_contents
                 WHERE meme_id = ?1
                 ORDER BY position",
            )?;
            statement
                .query_map([meme_id.to_string()], |row| {
                    Ok(RawContent {
                        id: uuid_from_column(row, 0)?,
                        kind: row.get(1)?,
                        text: row.get(2)?,
                        relative_path: row.get(3)?,
                        width: row.get(4)?,
                        height: row.get(5)?,
                        byte_size: row.get(6)?,
                        image_format: row.get(7)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        rows.into_iter()
            .map(|row| row.into_domain(&self.storage_root))
            .collect()
    }

    fn ensure_meme_pack(&self, id: Uuid) -> Result<()> {
        ensure_exists(&self.connection, "meme_packs", id)?
            .then_some(())
            .ok_or(Error::MemePackNotFound(id))
    }

    fn ensure_meme(&self, id: Uuid) -> Result<()> {
        ensure_exists(&self.connection, "memes", id)?
            .then_some(())
            .ok_or(Error::MemeNotFound(id))
    }

    fn ensure_tag(&self, id: Uuid) -> Result<()> {
        ensure_exists(&self.connection, "tags", id)?
            .then_some(())
            .ok_or(Error::TagNotFound(id))
    }

    fn tag_name_exists(&self, normalized_name: &str, except: Option<Uuid>) -> Result<bool> {
        let found = match except {
            Some(id) => self.connection.query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM tags WHERE normalized_name = ?1 AND id <> ?2
                 )",
                params![normalized_name, id.to_string()],
                |row| row.get(0),
            )?,
            None => self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM tags WHERE normalized_name = ?1)",
                [normalized_name],
                |row| row.get(0),
            )?,
        };
        Ok(found)
    }

    fn image_paths_for_meme(&self, meme_id: Uuid) -> Result<Vec<PathBuf>> {
        self.query_image_paths(
            "SELECT relative_path FROM meme_contents
             WHERE meme_id = ?1 AND kind = 'image'",
            meme_id,
        )
    }

    fn image_paths_for_meme_pack(&self, meme_pack_id: Uuid) -> Result<Vec<PathBuf>> {
        self.query_image_paths(
            "SELECT c.relative_path
             FROM meme_contents c
             JOIN memes m ON m.id = c.meme_id
             WHERE m.meme_pack_id = ?1 AND c.kind = 'image'",
            meme_pack_id,
        )
    }

    fn query_image_paths(&self, sql: &str, owner_id: Uuid) -> Result<Vec<PathBuf>> {
        let relative_paths = {
            let mut statement = self.connection.prepare(sql)?;
            statement
                .query_map([owner_id.to_string()], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?
        };
        relative_paths
            .into_iter()
            .map(|relative| {
                let absolute = resolve_media_path(&self.storage_root, Path::new(&relative))?;
                if !absolute.is_file() {
                    return Err(Error::MissingMedia(absolute));
                }
                Ok(absolute)
            })
            .collect()
    }

    fn clean_staging_directory(&self) -> Result<()> {
        let staging = self.storage_root.join(STAGING_DIRECTORY);
        for entry in fs::read_dir(staging)? {
            let path = entry?.path();
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        }
        Ok(())
    }

    fn collect_orphaned_media(&self) -> Result<()> {
        let referenced = {
            let mut statement = self
                .connection
                .prepare("SELECT relative_path FROM meme_contents WHERE kind = 'image'")?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<std::result::Result<HashSet<_>, _>>()?
        };
        let media_directory = self.storage_root.join(MEDIA_DIRECTORY);
        for entry in fs::read_dir(media_directory)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            let filename = entry.file_name();
            let filename = filename.to_str().ok_or_else(|| {
                Error::InvalidDatabase("managed media filename is not valid UTF-8".to_owned())
            })?;
            let relative = format!("{MEDIA_DIRECTORY}/{filename}");
            if !referenced.contains(&relative) {
                fs::remove_file(entry.path())?;
            }
        }
        Ok(())
    }
}

fn configure_connection(connection: &Connection) -> Result<()> {
    connection.pragma_update(None, "foreign_keys", true)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    let journal_mode: String =
        connection.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(Error::InvalidDatabase(format!(
            "SQLite refused WAL journal mode and returned `{journal_mode}`"
        )));
    }
    Ok(())
}

fn migrate_database(
    connection: &mut Connection,
    model_id: &str,
    embedding_dimension: usize,
) -> Result<()> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => create_schema(connection, model_id, embedding_dimension),
        SCHEMA_VERSION => validate_database_metadata(connection, model_id, embedding_dimension),
        actual => Err(Error::UnsupportedSchemaVersion {
            expected: SCHEMA_VERSION,
            actual,
        }),
    }
}

fn create_schema(
    connection: &mut Connection,
    model_id: &str,
    embedding_dimension: usize,
) -> Result<()> {
    let embedding_bytes = embedding_dimension
        .checked_mul(size_of::<f32>())
        .ok_or_else(|| Error::InvalidDatabase("embedding byte length exceeds usize".to_owned()))?;
    let transaction = connection.transaction()?;
    transaction.execute_batch(&format!(
        "CREATE TABLE metadata (
            singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
            schema_version INTEGER NOT NULL,
            embedding_model_id TEXT NOT NULL CHECK(trim(embedding_model_id) <> ''),
            embedding_dimension INTEGER NOT NULL CHECK(embedding_dimension > 0)
        );
        CREATE TABLE meme_packs (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL CHECK(trim(name) <> ''),
            name_embedding BLOB NOT NULL CHECK(length(name_embedding) = {embedding_bytes}),
            description TEXT,
            description_embedding BLOB,
            author TEXT,
            source TEXT,
            CHECK(
                (description IS NULL AND description_embedding IS NULL) OR
                (description IS NOT NULL AND trim(description) <> '' AND
                 description_embedding IS NOT NULL AND length(description_embedding) = {embedding_bytes})
            ),
            CHECK(author IS NULL OR trim(author) <> ''),
            CHECK(source IS NULL OR trim(source) <> '')
        );
        CREATE TABLE memes (
            id TEXT PRIMARY KEY,
            meme_pack_id TEXT NOT NULL REFERENCES meme_packs(id) ON DELETE CASCADE,
            name TEXT,
            name_embedding BLOB,
            description TEXT,
            description_embedding BLOB,
            CHECK(
                (name IS NULL AND name_embedding IS NULL) OR
                (name IS NOT NULL AND trim(name) <> '' AND
                 name_embedding IS NOT NULL AND length(name_embedding) = {embedding_bytes})
            ),
            CHECK(
                (description IS NULL AND description_embedding IS NULL) OR
                (description IS NOT NULL AND trim(description) <> '' AND
                 description_embedding IS NOT NULL AND length(description_embedding) = {embedding_bytes})
            )
        );
        CREATE INDEX memes_meme_pack ON memes(meme_pack_id);
        CREATE TABLE meme_contents (
            id TEXT PRIMARY KEY,
            meme_id TEXT NOT NULL REFERENCES memes(id) ON DELETE CASCADE,
            position INTEGER NOT NULL CHECK(position >= 0),
            kind TEXT NOT NULL CHECK(kind IN ('image', 'text')),
            text TEXT,
            relative_path TEXT UNIQUE,
            width INTEGER,
            height INTEGER,
            byte_size INTEGER,
            image_format TEXT CHECK(image_format IN ('png', 'jpeg', 'webp', 'gif')),
            embedding BLOB NOT NULL CHECK(length(embedding) = {embedding_bytes}),
            UNIQUE(meme_id, position),
            CHECK(
                (kind = 'text' AND text IS NOT NULL AND trim(text) <> '' AND
                 relative_path IS NULL AND width IS NULL AND height IS NULL AND
                 byte_size IS NULL AND image_format IS NULL) OR
                (kind = 'image' AND text IS NULL AND relative_path IS NOT NULL AND
                 width > 0 AND height > 0 AND byte_size > 0 AND image_format IS NOT NULL)
            )
        );
        CREATE INDEX meme_contents_meme ON meme_contents(meme_id, position);
        CREATE TABLE tags (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL CHECK(trim(name) <> ''),
            normalized_name TEXT NOT NULL UNIQUE CHECK(trim(normalized_name) <> ''),
            name_embedding BLOB NOT NULL CHECK(length(name_embedding) = {embedding_bytes})
        );
        CREATE TABLE meme_pack_tags (
            meme_pack_id TEXT NOT NULL REFERENCES meme_packs(id) ON DELETE CASCADE,
            tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY(meme_pack_id, tag_id)
        );
        CREATE INDEX meme_pack_tags_tag ON meme_pack_tags(tag_id);
        CREATE TABLE meme_tags (
            meme_id TEXT NOT NULL REFERENCES memes(id) ON DELETE CASCADE,
            tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY(meme_id, tag_id)
        );
        CREATE INDEX meme_tags_tag ON meme_tags(tag_id);"
    ))?;
    transaction.execute(
        "INSERT INTO metadata(singleton, schema_version, embedding_model_id, embedding_dimension)
         VALUES (1, ?1, ?2, ?3)",
        params![
            SCHEMA_VERSION,
            model_id,
            usize_to_i64(embedding_dimension, "embedding dimension")?
        ],
    )?;
    transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    transaction.commit()?;
    Ok(())
}

fn validate_database_metadata(
    connection: &Connection,
    model_id: &str,
    embedding_dimension: usize,
) -> Result<()> {
    let metadata = connection
        .query_row(
            "SELECT schema_version, embedding_model_id, embedding_dimension
             FROM metadata WHERE singleton = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            },
        )
        .optional()?
        .ok_or_else(|| Error::InvalidDatabase("metadata row is missing".to_owned()))?;
    if metadata.0 != SCHEMA_VERSION {
        return Err(Error::UnsupportedSchemaVersion {
            expected: SCHEMA_VERSION,
            actual: metadata.0,
        });
    }
    let stored_dimension = usize::try_from(metadata.2).map_err(|_| {
        Error::InvalidDatabase(format!(
            "stored embedding dimension {} is invalid",
            metadata.2
        ))
    })?;
    if metadata.1 != model_id || stored_dimension != embedding_dimension {
        return Err(Error::IncompatibleEmbeddingSpace {
            expected_model: metadata.1,
            expected_dimension: stored_dimension,
            actual_model: model_id.to_owned(),
            actual_dimension: embedding_dimension,
        });
    }
    Ok(())
}

#[derive(Debug)]
enum PreparedContent {
    Image {
        id: Uuid,
        source_path: PathBuf,
        relative_path: PathBuf,
        final_path: PathBuf,
        width: u32,
        height: u32,
        byte_size: u64,
        format: ImageFormat,
        embedding: Vec<u8>,
    },
    Text {
        id: Uuid,
        text: String,
        embedding: Vec<u8>,
    },
}

fn insert_contents(
    transaction: &Transaction<'_>,
    meme_id: Uuid,
    contents: &[PreparedContent],
) -> Result<()> {
    for (position, content) in contents.iter().enumerate() {
        let position = usize_to_i64(position, "Meme content position")?;
        match content {
            PreparedContent::Text {
                id,
                text,
                embedding,
            } => {
                transaction.execute(
                    "INSERT INTO meme_contents(
                        id, meme_id, position, kind, text, embedding
                     ) VALUES (?1, ?2, ?3, 'text', ?4, ?5)",
                    params![
                        id.to_string(),
                        meme_id.to_string(),
                        position,
                        text,
                        embedding,
                    ],
                )?;
            }
            PreparedContent::Image {
                id,
                relative_path,
                width,
                height,
                byte_size,
                format,
                embedding,
                ..
            } => {
                let relative_path = path_to_database_string(relative_path)?;
                transaction.execute(
                    "INSERT INTO meme_contents(
                        id, meme_id, position, kind, relative_path, width, height,
                        byte_size, image_format, embedding
                     ) VALUES (?1, ?2, ?3, 'image', ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        id.to_string(),
                        meme_id.to_string(),
                        position,
                        relative_path,
                        i64::from(*width),
                        i64::from(*height),
                        u64_to_i64(*byte_size, "image byte size")?,
                        format.as_database_str(),
                        embedding,
                    ],
                )?;
            }
        }
    }
    Ok(())
}

struct PendingFile {
    staged: PathBuf,
    final_path: PathBuf,
    promoted: bool,
}

struct PendingFiles {
    files: Vec<PendingFile>,
    active: bool,
}

impl PendingFiles {
    fn stage(staging_directory: &Path, contents: &[PreparedContent]) -> Result<Self> {
        let mut pending = Self {
            files: Vec::new(),
            active: true,
        };
        for content in contents {
            let PreparedContent::Image {
                id,
                source_path,
                final_path,
                ..
            } = content
            else {
                continue;
            };
            let staged = staging_directory.join(format!("{id}.tmp"));
            pending.files.push(PendingFile {
                staged: staged.clone(),
                final_path: final_path.clone(),
                promoted: false,
            });
            let mut source = File::open(source_path)?;
            let mut destination = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&staged)?;
            io::copy(&mut source, &mut destination)?;
            destination.sync_all()?;
        }
        Ok(pending)
    }

    fn promote(&mut self) -> Result<()> {
        for file in &mut self.files {
            if file.final_path.exists() {
                return Err(Error::InvalidDatabase(format!(
                    "managed media destination already exists: {}",
                    file.final_path.display()
                )));
            }
            fs::rename(&file.staged, &file.final_path)?;
            file.promoted = true;
        }
        Ok(())
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for PendingFiles {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        for file in &self.files {
            let path = if file.promoted {
                &file.final_path
            } else {
                &file.staged
            };
            let _ = fs::remove_file(path);
        }
    }
}

struct RawContent {
    id: Uuid,
    kind: String,
    text: Option<String>,
    relative_path: Option<String>,
    width: Option<i64>,
    height: Option<i64>,
    byte_size: Option<i64>,
    image_format: Option<String>,
}

impl RawContent {
    fn into_domain(self, storage_root: &Path) -> Result<MemeContent> {
        match self.kind.as_str() {
            "text" => {
                let text = self.text.ok_or_else(|| {
                    Error::InvalidDatabase(format!("text content {} has no text", self.id))
                })?;
                if self.relative_path.is_some()
                    || self.width.is_some()
                    || self.height.is_some()
                    || self.byte_size.is_some()
                    || self.image_format.is_some()
                {
                    return Err(Error::InvalidDatabase(format!(
                        "text content {} contains image metadata",
                        self.id
                    )));
                }
                Ok(MemeContent::Text(MemeText { id: self.id, text }))
            }
            "image" => {
                if self.text.is_some() {
                    return Err(Error::InvalidDatabase(format!(
                        "image content {} contains text",
                        self.id
                    )));
                }
                let relative_path = self.relative_path.ok_or_else(|| {
                    Error::InvalidDatabase(format!("image content {} has no path", self.id))
                })?;
                let relative_path = PathBuf::from(relative_path);
                let absolute_path = resolve_media_path(storage_root, &relative_path)?;
                if !absolute_path.is_file() {
                    return Err(Error::MissingMedia(absolute_path));
                }
                let width = positive_u32(self.width, self.id, "width")?;
                let height = positive_u32(self.height, self.id, "height")?;
                let byte_size = positive_u64(self.byte_size, self.id, "byte size")?;
                let format_raw = self.image_format.ok_or_else(|| {
                    Error::InvalidDatabase(format!("image content {} has no format", self.id))
                })?;
                let format = ImageFormat::from_database_str(&format_raw).ok_or_else(|| {
                    Error::InvalidDatabase(format!(
                        "image content {} has unknown format `{format_raw}`",
                        self.id
                    ))
                })?;
                Ok(MemeContent::Image(MemeImage {
                    id: self.id,
                    relative_path,
                    width,
                    height,
                    byte_size,
                    format,
                }))
            }
            kind => Err(Error::InvalidDatabase(format!(
                "content {} has unknown kind `{kind}`",
                self.id
            ))),
        }
    }
}

fn validate_meme_pack_input(input: NewMemePack) -> Result<NewMemePack> {
    Ok(NewMemePack {
        name: required_text(input.name, "MemePack name")?,
        description: optional_text(input.description, "MemePack description")?,
        author: optional_text(input.author, "MemePack author")?,
        source: optional_text(input.source, "MemePack source")?,
    })
}

fn validate_update_meme_pack_input(input: UpdateMemePack) -> Result<UpdateMemePack> {
    Ok(UpdateMemePack {
        name: required_text(input.name, "MemePack name")?,
        description: optional_text(input.description, "MemePack description")?,
        author: optional_text(input.author, "MemePack author")?,
        source: optional_text(input.source, "MemePack source")?,
    })
}

fn validate_meme_metadata(
    name: Option<String>,
    description: Option<String>,
) -> Result<(Option<String>, Option<String>)> {
    Ok((
        optional_text(name, "Meme name")?,
        optional_text(description, "Meme description")?,
    ))
}

fn required_text(value: String, field: &'static str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::EmptyField { field });
    }
    Ok(trimmed.to_owned())
}

fn optional_text(value: Option<String>, field: &'static str) -> Result<Option<String>> {
    value.map(|value| required_text(value, field)).transpose()
}

fn normalize_tag_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn encode_embedding(
    field: &'static str,
    values: Vec<f32>,
    expected_dimension: usize,
) -> Result<Vec<u8>> {
    if values.len() != expected_dimension {
        return Err(Error::InvalidEmbeddingDimension {
            field,
            expected: expected_dimension,
            actual: values.len(),
        });
    }
    if let Some((index, _)) = values
        .iter()
        .enumerate()
        .find(|(_, value)| !value.is_finite())
    {
        return Err(Error::NonFiniteEmbedding { field, index });
    }
    let squared_norm = values
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>();
    if !squared_norm.is_finite() || squared_norm <= f64::EPSILON {
        return Err(Error::ZeroEmbedding { field });
    }
    let mut encoded = Vec::with_capacity(expected_dimension * size_of::<f32>());
    for value in values {
        encoded.extend_from_slice(&value.to_le_bytes());
    }
    Ok(encoded)
}

fn decode_supported_image(path: &Path) -> Result<(ImageFormat, DynamicImage)> {
    let reader = ImageReader::open(path)?.with_guessed_format()?;
    let detected = reader
        .format()
        .ok_or_else(|| Error::UnsupportedImageFormat(path.to_path_buf()))?;
    let format = match detected {
        DecodedImageFormat::Png => ImageFormat::Png,
        DecodedImageFormat::Jpeg => ImageFormat::Jpeg,
        DecodedImageFormat::WebP => ImageFormat::WebP,
        DecodedImageFormat::Gif => ImageFormat::Gif,
        _ => return Err(Error::UnsupportedImageFormat(path.to_path_buf())),
    };
    Ok((format, reader.decode()?))
}

fn resolve_media_path(storage_root: &Path, relative_path: &Path) -> Result<PathBuf> {
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(Error::UnsafeMediaPath(relative_path.to_path_buf()));
    }
    let parent = relative_path
        .parent()
        .ok_or_else(|| Error::UnsafeMediaPath(relative_path.to_path_buf()))?;
    if parent != Path::new(MEDIA_DIRECTORY) || relative_path.file_name().is_none() {
        return Err(Error::UnsafeMediaPath(relative_path.to_path_buf()));
    }
    Ok(storage_root.join(relative_path))
}

fn path_to_database_string(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(
                part.to_str()
                    .ok_or_else(|| Error::UnsafeMediaPath(path.to_path_buf()))?,
            ),
            _ => return Err(Error::UnsafeMediaPath(path.to_path_buf())),
        }
    }
    Ok(parts.join("/"))
}

fn ensure_exists(connection: &Connection, table: &str, id: Uuid) -> Result<bool> {
    let sql = format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE id = ?1)");
    connection
        .query_row(&sql, [id.to_string()], |row| row.get(0))
        .map_err(Error::from)
}

fn insert_tag_association(
    connection: &Connection,
    table: &'static str,
    owner_column: &'static str,
    target: &'static str,
    target_id: Uuid,
    tag_id: Uuid,
) -> Result<()> {
    let exists_sql = format!(
        "SELECT EXISTS(
            SELECT 1 FROM {table} WHERE {owner_column} = ?1 AND tag_id = ?2
         )"
    );
    let exists: bool = connection.query_row(
        &exists_sql,
        params![target_id.to_string(), tag_id.to_string()],
        |row| row.get(0),
    )?;
    if exists {
        return Err(Error::TagAssociationExists {
            target,
            target_id,
            tag_id,
        });
    }
    let insert_sql = format!("INSERT INTO {table}({owner_column}, tag_id) VALUES (?1, ?2)");
    connection.execute(
        &insert_sql,
        params![target_id.to_string(), tag_id.to_string()],
    )?;
    Ok(())
}

fn delete_tag_association(
    connection: &Connection,
    table: &'static str,
    owner_column: &'static str,
    target: &'static str,
    target_id: Uuid,
    tag_id: Uuid,
) -> Result<()> {
    let sql = format!("DELETE FROM {table} WHERE {owner_column} = ?1 AND tag_id = ?2");
    let changed = connection.execute(&sql, params![target_id.to_string(), tag_id.to_string()])?;
    if changed == 0 {
        return Err(Error::TagAssociationNotFound {
            target,
            target_id,
            tag_id,
        });
    }
    Ok(())
}

fn map_meme_pack(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemePack> {
    Ok(MemePack {
        id: uuid_from_column(row, 0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        author: row.get(3)?,
        source: row.get(4)?,
    })
}

fn map_tag(row: &rusqlite::Row<'_>) -> rusqlite::Result<Tag> {
    Ok(Tag {
        id: uuid_from_column(row, 0)?,
        name: row.get(1)?,
    })
}

fn uuid_from_column(row: &rusqlite::Row<'_>, column: usize) -> rusqlite::Result<Uuid> {
    let raw: String = row.get(column)?;
    Uuid::parse_str(&raw).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            column,
            rusqlite::types::Type::Text,
            Box::new(error),
        )
    })
}

fn positive_u32(value: Option<i64>, id: Uuid, field: &str) -> Result<u32> {
    let raw = value
        .ok_or_else(|| Error::InvalidDatabase(format!("image content {id} has no {field}")))?;
    let converted = u32::try_from(raw).map_err(|_| {
        Error::InvalidDatabase(format!("image content {id} has invalid {field} {raw}"))
    })?;
    if converted == 0 {
        return Err(Error::InvalidDatabase(format!(
            "image content {id} has zero {field}"
        )));
    }
    Ok(converted)
}

fn positive_u64(value: Option<i64>, id: Uuid, field: &str) -> Result<u64> {
    let raw = value
        .ok_or_else(|| Error::InvalidDatabase(format!("image content {id} has no {field}")))?;
    let converted = u64::try_from(raw).map_err(|_| {
        Error::InvalidDatabase(format!("image content {id} has invalid {field} {raw}"))
    })?;
    if converted == 0 {
        return Err(Error::InvalidDatabase(format!(
            "image content {id} has zero {field}"
        )));
    }
    Ok(converted)
}

fn usize_to_i64(value: usize, field: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| {
        Error::InvalidDatabase(format!("{field} exceeds SQLite integer range: {value}"))
    })
}

fn u64_to_i64(value: u64, field: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| {
        Error::InvalidDatabase(format!("{field} exceeds SQLite integer range: {value}"))
    })
}

fn remove_managed_files(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        fs::remove_file(path)?;
    }
    Ok(())
}
