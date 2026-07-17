use std::{collections::HashMap, num::NonZeroUsize};

use bytemuck::cast_slice;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use uuid::Uuid;

use crate::{Bundle, Error, FeatureSchema, Result, sqlite_vec};

const DATABASE_SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DatabaseSync {
    Initialized,
    Updated { previous_revision: u64 },
    Unchanged,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StoredMatch {
    pub character_id: Uuid,
    pub name: String,
    pub distance: f32,
}

pub struct WaifuDatabase {
    connection: Connection,
    feature_schema: FeatureSchema,
}

impl std::fmt::Debug for WaifuDatabase {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WaifuDatabase")
            .field("feature_schema", &self.feature_schema.id)
            .finish_non_exhaustive()
    }
}

impl WaifuDatabase {
    pub fn open(mut connection: Connection, bundle: &Bundle) -> Result<(Self, DatabaseSync)> {
        bundle.validate()?;
        sqlite_vec::initialize(&connection)?;
        create_schema(&connection, bundle.feature_schema.dimension())?;
        let sync = sync_bundle(&mut connection, bundle)?;
        Ok((
            Self {
                connection,
                feature_schema: bundle.feature_schema.clone(),
            },
            sync,
        ))
    }

    pub fn feature_schema(&self) -> &FeatureSchema {
        &self.feature_schema
    }

    pub fn into_connection(self) -> Connection {
        self.connection
    }

    pub fn character_count(&self) -> Result<usize> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM waifu_sensor_characters WHERE active = 1",
            [],
            |row| row.get(0),
        )?;
        usize::try_from(count).map_err(|_| {
            Error::IncompatibleDatabase(format!("negative character count returned: {count}"))
        })
    }

    pub(crate) fn character_id_by_name(&self, name: &str) -> Result<Uuid> {
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM waifu_sensor_characters
                 WHERE canonical_name = ?1 COLLATE NOCASE AND active = 1",
                [name],
                |row| row.get(0),
            )
            .optional()?;
        let id = id.ok_or_else(|| Error::CharacterNameNotFound(name.to_owned()))?;
        Uuid::parse_str(&id).map_err(|error| {
            Error::IncompatibleDatabase(format!(
                "character `{name}` has invalid UUID `{id}`: {error}"
            ))
        })
    }

    pub(crate) fn insert_character(
        &mut self,
        name: &str,
        samples: &[Vec<f32>],
        prototypes: &[Vec<f32>],
    ) -> Result<Uuid> {
        if name.trim().is_empty() {
            return Err(Error::EmptyCharacterName);
        }
        if samples.is_empty() || prototypes.is_empty() {
            return Err(Error::EmptyReferenceImages);
        }
        for vector in samples.iter().chain(prototypes) {
            self.feature_schema.validate_vector(vector)?;
        }
        let duplicate = self
            .connection
            .query_row(
                "SELECT 1 FROM waifu_sensor_characters WHERE canonical_name = ?1 COLLATE NOCASE",
                [name],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if duplicate {
            return Err(Error::DuplicateCharacter(name.to_owned()));
        }

        let id = Uuid::new_v4();
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO waifu_sensor_characters(id, canonical_name, origin, active) VALUES (?1, ?2, 'user', 1)",
            params![id.to_string(), name],
        )?;
        insert_samples(&transaction, id, samples)?;
        insert_prototypes(&transaction, &self.feature_schema, id, "user", prototypes)?;
        transaction.commit()?;
        Ok(id)
    }

    pub(crate) fn append_character_vectors(
        &mut self,
        character_id: Uuid,
        new_samples: &[Vec<f32>],
        all_prototypes: &[Vec<f32>],
    ) -> Result<()> {
        if new_samples.is_empty() {
            return Err(Error::EmptyReferenceImages);
        }
        for vector in new_samples.iter().chain(all_prototypes) {
            self.feature_schema.validate_vector(vector)?;
        }
        self.ensure_character(character_id)?;

        let transaction = self.connection.transaction()?;
        insert_samples(&transaction, character_id, new_samples)?;
        let mut statement = transaction.prepare(
            "SELECT rowid FROM waifu_sensor_prototypes WHERE character_id = ?1 AND origin = 'user'",
        )?;
        let row_ids: Vec<i64> = statement
            .query_map([character_id.to_string()], |row| row.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        drop(statement);
        for row_id in row_ids {
            transaction.execute(
                "DELETE FROM waifu_sensor_prototype_vectors WHERE rowid = ?1",
                [row_id],
            )?;
        }
        transaction.execute(
            "DELETE FROM waifu_sensor_prototypes WHERE character_id = ?1 AND origin = 'user'",
            [character_id.to_string()],
        )?;
        insert_prototypes(
            &transaction,
            &self.feature_schema,
            character_id,
            "user",
            all_prototypes,
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub(crate) fn samples_for_character(&self, character_id: Uuid) -> Result<Vec<Vec<f32>>> {
        self.ensure_character(character_id)?;
        let mut statement = self.connection.prepare(
            "SELECT feature_vector FROM waifu_sensor_reference_samples WHERE character_id = ?1 ORDER BY rowid",
        )?;
        statement
            .query_map([character_id.to_string()], |row| row.get::<_, Vec<u8>>(0))?
            .map(|item| decode_vector(&item?, self.feature_schema.dimension()))
            .collect()
    }

    pub(crate) fn search(
        &self,
        weighted_query: &[f32],
        top_n: NonZeroUsize,
    ) -> Result<Vec<StoredMatch>> {
        if weighted_query.len() != self.feature_schema.dimension() {
            return Err(Error::InvalidVectorDimension {
                expected: self.feature_schema.dimension(),
                actual: weighted_query.len(),
            });
        }

        let prototype_count_raw: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM waifu_sensor_prototypes p JOIN waifu_sensor_characters c ON c.id = p.character_id WHERE p.active = 1 AND c.active = 1",
            [],
            |row| row.get(0),
        )?;
        let prototype_count = usize::try_from(prototype_count_raw).map_err(|_| {
            Error::IncompatibleDatabase(format!(
                "negative prototype count returned: {prototype_count_raw}"
            ))
        })?;
        if prototype_count == 0 {
            return Ok(Vec::new());
        }

        let mut limit = top_n.get().min(prototype_count).max(1);
        loop {
            let mut statement = self.connection.prepare(
                "SELECT p.character_id, c.canonical_name, nearest.distance
                 FROM (
                     SELECT rowid, distance
                     FROM waifu_sensor_prototype_vectors
                     WHERE embedding MATCH ?1 AND k = ?2
                 ) AS nearest
                 JOIN waifu_sensor_prototypes p ON p.rowid = nearest.rowid
                 JOIN waifu_sensor_characters c ON c.id = p.character_id
                 WHERE p.active = 1 AND c.active = 1
                 ORDER BY nearest.distance",
            )?;
            let candidates = statement
                .query_map(params![cast_slice(weighted_query), limit as i64], |row| {
                    let id: String = row.get(0)?;
                    let parsed = Uuid::parse_str(&id).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Ok(StoredMatch {
                        character_id: parsed,
                        name: row.get(1)?,
                        distance: row.get(2)?,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            let mut unique = HashMap::<Uuid, StoredMatch>::new();
            for candidate in candidates {
                unique.entry(candidate.character_id).or_insert(candidate);
            }
            if unique.len() >= top_n.get() || limit == prototype_count {
                let mut matches: Vec<_> = unique.into_values().collect();
                matches.sort_by(|left, right| left.distance.total_cmp(&right.distance));
                matches.truncate(top_n.get());
                return Ok(matches);
            }
            limit = (limit * 2).min(prototype_count);
        }
    }

    pub(crate) fn prototypes_for_character(&self, character_id: Uuid) -> Result<Vec<Vec<f32>>> {
        self.ensure_character(character_id)?;
        let mut statement = self.connection.prepare(
            "SELECT feature_vector FROM waifu_sensor_prototypes WHERE character_id = ?1 AND active = 1 ORDER BY rowid",
        )?;
        statement
            .query_map([character_id.to_string()], |row| row.get::<_, Vec<u8>>(0))?
            .map(|item| decode_vector(&item?, self.feature_schema.dimension()))
            .collect()
    }

    fn ensure_character(&self, character_id: Uuid) -> Result<()> {
        let exists = self
            .connection
            .query_row(
                "SELECT 1 FROM waifu_sensor_characters WHERE id = ?1 AND active = 1",
                [character_id.to_string()],
                |_| Ok(()),
            )
            .optional()?
            .is_some();
        if !exists {
            return Err(Error::CharacterNotFound(character_id));
        }
        Ok(())
    }
}

fn create_schema(connection: &Connection, dimension: usize) -> Result<()> {
    let existing_version: Option<i64> = connection
        .query_row(
            "SELECT schema_version FROM waifu_sensor_meta WHERE singleton = 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .or_else(|error| match error {
            rusqlite::Error::SqliteFailure(_, Some(message))
                if message.contains("no such table") =>
            {
                Ok(None)
            }
            other => Err(other),
        })?;
    if let Some(version) = existing_version {
        if version != DATABASE_SCHEMA_VERSION {
            return Err(Error::IncompatibleDatabase(format!(
                "expected schema version {DATABASE_SCHEMA_VERSION}, got {version}"
            )));
        }
        return Ok(());
    }

    connection.execute_batch(&format!(
        "CREATE TABLE waifu_sensor_meta (
            singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
            schema_version INTEGER NOT NULL,
            bundle_id TEXT,
            bundle_revision INTEGER,
            bundle_digest TEXT,
            feature_schema_id TEXT NOT NULL,
            feature_schema_digest TEXT NOT NULL
        );
        CREATE TABLE waifu_sensor_characters (
            id TEXT PRIMARY KEY,
            canonical_name TEXT NOT NULL UNIQUE COLLATE NOCASE,
            origin TEXT NOT NULL CHECK(origin IN ('bundle', 'user')),
            active INTEGER NOT NULL CHECK(active IN (0, 1))
        );
        CREATE TABLE waifu_sensor_reference_samples (
            rowid INTEGER PRIMARY KEY,
            character_id TEXT NOT NULL REFERENCES waifu_sensor_characters(id) ON DELETE CASCADE,
            feature_vector BLOB NOT NULL
        );
        CREATE TABLE waifu_sensor_prototypes (
            rowid INTEGER PRIMARY KEY,
            stable_key TEXT UNIQUE,
            character_id TEXT NOT NULL REFERENCES waifu_sensor_characters(id) ON DELETE CASCADE,
            origin TEXT NOT NULL CHECK(origin IN ('bundle', 'user')),
            active INTEGER NOT NULL CHECK(active IN (0, 1)),
            feature_vector BLOB NOT NULL
        );
        CREATE INDEX waifu_sensor_prototypes_character ON waifu_sensor_prototypes(character_id);
        CREATE VIRTUAL TABLE waifu_sensor_prototype_vectors USING vec0(embedding float[{dimension}]);"
    ))?;
    Ok(())
}

fn sync_bundle(connection: &mut Connection, bundle: &Bundle) -> Result<DatabaseSync> {
    let schema_digest = bundle.feature_schema.digest()?;
    let meta: Option<(String, i64, String, String, String)> = connection
        .query_row(
            "SELECT bundle_id, bundle_revision, bundle_digest, feature_schema_id, feature_schema_digest FROM waifu_sensor_meta WHERE singleton = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        )
        .optional()?;

    if let Some((_, revision, digest, database_schema, database_schema_digest)) = &meta {
        if database_schema != &bundle.feature_schema.id || database_schema_digest != &schema_digest
        {
            return Err(Error::IncompatibleBundle {
                bundle_schema: format!("{} ({schema_digest})", bundle.feature_schema.id),
                database_schema: format!("{database_schema} ({database_schema_digest})"),
            });
        }
        let bundle_revision = i64::try_from(bundle.manifest.revision).map_err(|_| {
            Error::InvalidBundle(format!(
                "bundle revision {} exceeds SQLite integer range",
                bundle.manifest.revision
            ))
        })?;
        if *revision == bundle_revision && digest == &bundle.content_digest {
            return Ok(DatabaseSync::Unchanged);
        }
    }

    let transaction = connection.transaction()?;
    if meta.is_none() {
        transaction.execute(
            "INSERT INTO waifu_sensor_meta(singleton, schema_version, feature_schema_id, feature_schema_digest) VALUES (1, ?1, ?2, ?3)",
            params![DATABASE_SCHEMA_VERSION, bundle.feature_schema.id, schema_digest],
        )?;
    }

    let mut old_bundle_rows = transaction.prepare(
        "SELECT rowid FROM waifu_sensor_prototypes WHERE origin = 'bundle' AND active = 1",
    )?;
    let old_row_ids: Vec<i64> = old_bundle_rows
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<_, _>>()?;
    drop(old_bundle_rows);
    for row_id in old_row_ids {
        transaction.execute(
            "DELETE FROM waifu_sensor_prototype_vectors WHERE rowid = ?1",
            [row_id],
        )?;
    }
    transaction.execute(
        "UPDATE waifu_sensor_prototypes SET active = 0 WHERE origin = 'bundle'",
        [],
    )?;
    transaction.execute(
        "UPDATE waifu_sensor_characters SET active = 0 WHERE origin = 'bundle'",
        [],
    )?;

    for character in &bundle.characters {
        let existing_id: Option<String> = transaction
            .query_row(
                "SELECT id FROM waifu_sensor_characters WHERE canonical_name = ?1 COLLATE NOCASE",
                [&character.name],
                |row| row.get(0),
            )
            .optional()?;
        let character_id = match existing_id {
            Some(id) => {
                let id = Uuid::parse_str(&id).map_err(|error| {
                    Error::IncompatibleDatabase(format!(
                        "character `{}` has invalid UUID `{id}`: {error}",
                        character.name
                    ))
                })?;
                transaction.execute(
                    "UPDATE waifu_sensor_characters SET active = 1 WHERE id = ?1",
                    [id.to_string()],
                )?;
                id
            }
            None => {
                let id = bundle.character_id(&character.name);
                transaction.execute(
                    "INSERT INTO waifu_sensor_characters(id, canonical_name, origin, active)
                     VALUES (?1, ?2, 'bundle', 1)",
                    params![id.to_string(), character.name],
                )?;
                id
            }
        };
        for (index, vector) in character.prototypes.iter().enumerate() {
            let stable_key = format!("{}:{}:{index}", bundle.manifest.bundle_id, character.name);
            upsert_bundle_prototype(
                &transaction,
                &bundle.feature_schema,
                character_id,
                &stable_key,
                vector,
            )?;
        }
    }

    transaction.execute(
        "UPDATE waifu_sensor_characters
         SET active = 1
         WHERE EXISTS (
             SELECT 1 FROM waifu_sensor_prototypes p
             WHERE p.character_id = waifu_sensor_characters.id AND p.active = 1
         )",
        [],
    )?;
    transaction.execute(
        "UPDATE waifu_sensor_meta SET bundle_id = ?1, bundle_revision = ?2, bundle_digest = ?3,
         feature_schema_id = ?4, feature_schema_digest = ?5 WHERE singleton = 1",
        params![
            bundle.manifest.bundle_id.to_string(),
            i64::try_from(bundle.manifest.revision).map_err(|_| Error::InvalidBundle(format!(
                "bundle revision {} exceeds SQLite integer range",
                bundle.manifest.revision
            )))?,
            bundle.content_digest,
            bundle.feature_schema.id,
            schema_digest
        ],
    )?;
    transaction.commit()?;

    Ok(match meta {
        Some((_, revision, _, _, _)) => DatabaseSync::Updated {
            previous_revision: u64::try_from(revision).map_err(|_| {
                Error::IncompatibleDatabase(format!("negative bundle revision: {revision}"))
            })?,
        },
        None => DatabaseSync::Initialized,
    })
}

fn insert_samples(
    transaction: &Transaction<'_>,
    character_id: Uuid,
    samples: &[Vec<f32>],
) -> Result<()> {
    let mut statement = transaction.prepare(
        "INSERT INTO waifu_sensor_reference_samples(character_id, feature_vector) VALUES (?1, ?2)",
    )?;
    for sample in samples {
        statement.execute(params![character_id.to_string(), cast_slice(sample)])?;
    }
    Ok(())
}

fn insert_prototypes(
    transaction: &Transaction<'_>,
    schema: &FeatureSchema,
    character_id: Uuid,
    origin: &str,
    prototypes: &[Vec<f32>],
) -> Result<()> {
    for prototype in prototypes {
        transaction.execute(
            "INSERT INTO waifu_sensor_prototypes(stable_key, character_id, origin, active, feature_vector)
             VALUES (NULL, ?1, ?2, 1, ?3)",
            params![character_id.to_string(), origin, cast_slice(prototype)],
        )?;
        let row_id = transaction.last_insert_rowid();
        let weighted = schema.weighted(prototype)?;
        transaction.execute(
            "INSERT INTO waifu_sensor_prototype_vectors(rowid, embedding) VALUES (?1, ?2)",
            params![row_id, cast_slice(&weighted)],
        )?;
    }
    Ok(())
}

fn upsert_bundle_prototype(
    transaction: &Transaction<'_>,
    schema: &FeatureSchema,
    character_id: Uuid,
    stable_key: &str,
    vector: &[f32],
) -> Result<()> {
    transaction.execute(
        "INSERT INTO waifu_sensor_prototypes(stable_key, character_id, origin, active, feature_vector)
         VALUES (?1, ?2, 'bundle', 1, ?3)
         ON CONFLICT(stable_key) DO UPDATE SET character_id = excluded.character_id,
             active = 1, feature_vector = excluded.feature_vector",
        params![stable_key, character_id.to_string(), cast_slice(vector)],
    )?;
    let row_id: i64 = transaction.query_row(
        "SELECT rowid FROM waifu_sensor_prototypes WHERE stable_key = ?1",
        [stable_key],
        |row| row.get(0),
    )?;
    let weighted = schema.weighted(vector)?;
    transaction.execute(
        "DELETE FROM waifu_sensor_prototype_vectors WHERE rowid = ?1",
        [row_id],
    )?;
    transaction.execute(
        "INSERT INTO waifu_sensor_prototype_vectors(rowid, embedding) VALUES (?1, ?2)",
        params![row_id, cast_slice(&weighted)],
    )?;
    Ok(())
}

fn decode_vector(bytes: &[u8], dimension: usize) -> Result<Vec<f32>> {
    if bytes.len() != dimension * std::mem::size_of::<f32>() {
        return Err(Error::InvalidVectorDimension {
            expected: dimension,
            actual: bytes.len() / std::mem::size_of::<f32>(),
        });
    }
    let (chunks, remainder) = bytes.as_chunks::<4>();
    debug_assert!(remainder.is_empty());
    Ok(chunks
        .iter()
        .map(|chunk| f32::from_ne_bytes(*chunk))
        .collect())
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use rusqlite::Connection;
    use uuid::Uuid;

    use crate::{Bundle, BundleCharacter, BundleManifest, Feature, FeatureSchema};

    use super::{DatabaseSync, WaifuDatabase};

    fn schema() -> FeatureSchema {
        FeatureSchema {
            id: "test-v1".to_owned(),
            model_id: "test-model".to_owned(),
            threshold: 0.4,
            features: vec![
                Feature {
                    tag: "red".to_owned(),
                    weight: 1.0,
                },
                Feature {
                    tag: "blue".to_owned(),
                    weight: 0.5,
                },
            ],
        }
    }

    fn bundle(revision: u64, characters: Vec<(&str, Vec<Vec<f32>>)>) -> Bundle {
        Bundle {
            manifest: BundleManifest {
                bundle_id: Uuid::parse_str("31fb25a3-c194-45eb-b54d-cd7691c25eed").unwrap(),
                revision,
                source: "test".to_owned(),
                feature_schema: "features.json".into(),
                characters: "characters.json".into(),
                characters_sha256: "unused-in-memory".to_owned(),
            },
            feature_schema: schema(),
            characters: characters
                .into_iter()
                .map(|(name, prototypes)| BundleCharacter {
                    name: name.to_owned(),
                    prototypes,
                })
                .collect(),
            content_digest: format!("revision-{revision}"),
        }
    }

    #[test]
    fn initializes_inside_an_existing_database_and_returns_unique_characters() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute("CREATE TABLE application_data(value TEXT)", [])
            .unwrap();
        let bundle = bundle(
            1,
            vec![
                ("red", vec![vec![1.0, 0.0], vec![0.9, 0.0]]),
                ("blue", vec![vec![0.0, 1.0]]),
            ],
        );

        let (database, sync) = WaifuDatabase::open(connection, &bundle).unwrap();
        let matches = database
            .search(&[1.0, 0.0], NonZeroUsize::new(2).unwrap())
            .unwrap();

        assert_eq!(sync, DatabaseSync::Initialized);
        assert_eq!(database.character_count().unwrap(), 2);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].name, "red");
        assert_eq!(matches[1].name, "blue");
        let connection = database.into_connection();
        let application_table: String = connection
            .query_row(
                "SELECT name FROM sqlite_master WHERE name = 'application_data'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(application_table, "application_data");
    }

    #[test]
    fn bundle_update_preserves_user_characters_and_is_idempotent() {
        let first = bundle(1, vec![("builtin", vec![vec![0.8, 0.2]])]);
        let (mut database, _) =
            WaifuDatabase::open(Connection::open_in_memory().unwrap(), &first).unwrap();
        let user_id = database
            .insert_character("user", &[vec![0.2, 0.8]], &[vec![0.2, 0.8]])
            .unwrap();
        let second = bundle(2, vec![("builtin", vec![vec![0.9, 0.1]])]);

        let (database, sync) = WaifuDatabase::open(database.into_connection(), &second).unwrap();

        assert_eq!(
            sync,
            DatabaseSync::Updated {
                previous_revision: 1
            }
        );
        assert_eq!(database.character_count().unwrap(), 2);
        assert_eq!(
            database.prototypes_for_character(user_id).unwrap(),
            vec![vec![0.2, 0.8]]
        );

        let (database, sync) = WaifuDatabase::open(database.into_connection(), &second).unwrap();
        assert_eq!(sync, DatabaseSync::Unchanged);
        assert_eq!(database.character_count().unwrap(), 2);
    }

    #[test]
    fn bundle_update_merges_a_new_builtin_with_an_existing_user_character_name() {
        let first = bundle(1, vec![("first builtin", vec![vec![0.8, 0.2]])]);
        let (mut database, _) =
            WaifuDatabase::open(Connection::open_in_memory().unwrap(), &first).unwrap();
        let user_id = database
            .insert_character("future builtin", &[vec![0.2, 0.8]], &[vec![0.2, 0.8]])
            .unwrap();
        let second = bundle(
            2,
            vec![
                ("first builtin", vec![vec![0.8, 0.2]]),
                ("future builtin", vec![vec![0.1, 0.9]]),
            ],
        );

        let (database, _) = WaifuDatabase::open(database.into_connection(), &second).unwrap();
        let prototypes = database.prototypes_for_character(user_id).unwrap();

        assert_eq!(database.character_count().unwrap(), 2);
        assert_eq!(prototypes, vec![vec![0.2, 0.8], vec![0.1, 0.9]]);
    }
}
