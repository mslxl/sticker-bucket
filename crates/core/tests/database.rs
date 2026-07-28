use std::{
    fs::{self, File},
    io,
    path::Path,
};

use image::{
    Delay, DynamicImage, Frame, GenericImageView, ImageBuffer, ImageFormat as EncodedImageFormat,
    Rgba, RgbaImage, codecs::gif::GifEncoder,
};
use memelith_core::{
    EffectiveTag, EmbeddingProvider, EmbeddingProviderError, Error, ImageFormat, MemeContent,
    MemeDatabase, NewMeme, NewMemeContent, NewMemePack, NewTag, UpdateMemeMetadata, UpdateMemePack,
};
use rusqlite::Connection;

#[derive(Clone)]
struct FakeEmbeddingProvider {
    model_id: String,
    dimension: usize,
    wrong_output_dimension: Option<usize>,
    fail: bool,
}

impl FakeEmbeddingProvider {
    fn valid() -> Self {
        Self {
            model_id: "test-embedding-v1".to_owned(),
            dimension: 4,
            wrong_output_dimension: None,
            fail: false,
        }
    }

    fn text_values(text: &str, dimension: usize) -> Vec<f32> {
        let mut values = (1..=dimension)
            .map(|value| value as f32)
            .collect::<Vec<_>>();
        if let Some(first) = values.first_mut() {
            *first = text.len() as f32 + 1.0;
        }
        values
    }

    fn image_values(image: &DynamicImage, dimension: usize) -> Vec<f32> {
        let pixel = image.get_pixel(0, 0).0;
        let base = [
            image.width() as f32,
            image.height() as f32,
            f32::from(pixel[0]) + 1.0,
            f32::from(pixel[2]) + 1.0,
        ];
        (0..dimension)
            .map(|index| base[index % base.len()])
            .collect()
    }

    fn output_dimension(&self) -> usize {
        self.wrong_output_dimension.unwrap_or(self.dimension)
    }
}

impl EmbeddingProvider for FakeEmbeddingProvider {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn embed_text(&mut self, text: &str) -> std::result::Result<Vec<f32>, EmbeddingProviderError> {
        if self.fail {
            return Err(Box::new(io::Error::other("intentional provider failure")));
        }
        Ok(Self::text_values(text, self.output_dimension()))
    }

    fn embed_image(
        &mut self,
        image: &DynamicImage,
    ) -> std::result::Result<Vec<f32>, EmbeddingProviderError> {
        if self.fail {
            return Err(Box::new(io::Error::other("intentional provider failure")));
        }
        Ok(Self::image_values(image, self.output_dimension()))
    }
}

struct FixedVectorProvider {
    values: Vec<f32>,
}

impl EmbeddingProvider for FixedVectorProvider {
    fn model_id(&self) -> &str {
        "fixed-vector-test"
    }

    fn dimension(&self) -> usize {
        self.values.len()
    }

    fn embed_text(&mut self, _text: &str) -> std::result::Result<Vec<f32>, EmbeddingProviderError> {
        Ok(self.values.clone())
    }

    fn embed_image(
        &mut self,
        _image: &DynamicImage,
    ) -> std::result::Result<Vec<f32>, EmbeddingProviderError> {
        Ok(self.values.clone())
    }
}

#[test]
fn persists_mixed_contents_embeddings_and_full_meme_lifecycle() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("source.png");
    write_static_image(&source, EncodedImageFormat::Png, 3, 2, [220, 10, 30, 255]);
    let storage = directory.path().join("library");
    let mut database = MemeDatabase::open(&storage, FakeEmbeddingProvider::valid()).unwrap();

    let pack = database
        .create_meme_pack(NewMemePack {
            name: "  Reactions  ".to_owned(),
            description: Some("  Everyday reactions  ".to_owned()),
            author: Some("  Alice  ".to_owned()),
            source: Some("  local import  ".to_owned()),
        })
        .unwrap();
    assert_eq!(pack.name, "Reactions");
    assert_eq!(pack.description.as_deref(), Some("Everyday reactions"));
    assert_eq!(pack.author.as_deref(), Some("Alice"));
    assert_eq!(pack.source.as_deref(), Some("local import"));

    let meme = database
        .create_meme(
            pack.id,
            NewMeme {
                name: Some("  Greeting  ".to_owned()),
                description: Some("  A mixed Meme  ".to_owned()),
                contents: vec![
                    NewMemeContent::Text {
                        text: "  hello  ".to_owned(),
                    },
                    NewMemeContent::Image {
                        source_path: source.clone(),
                    },
                ],
            },
        )
        .unwrap();
    assert_eq!(meme.name.as_deref(), Some("Greeting"));
    assert_eq!(meme.description.as_deref(), Some("A mixed Meme"));
    assert_eq!(meme.contents.len(), 2);
    assert!(matches!(
        &meme.contents[0],
        MemeContent::Text(text) if text.text == "hello"
    ));
    let image = match &meme.contents[1] {
        MemeContent::Image(image) => image,
        other => panic!("expected image content, got {other:?}"),
    };
    assert!(!image.relative_path.is_absolute());
    assert_eq!(
        image.relative_path.parent(),
        Some(Path::new("media/images"))
    );
    assert_eq!((image.width, image.height), (3, 2));
    assert_eq!(image.format, ImageFormat::Png);
    assert_eq!(image.byte_size, fs::metadata(&source).unwrap().len());
    let managed_path = database.resolve_media_path(&image.relative_path).unwrap();
    assert_eq!(fs::read(&managed_path).unwrap(), fs::read(&source).unwrap());

    let raw = Connection::open(database.database_path()).unwrap();
    let stored_pack_name: Vec<u8> = raw
        .query_row(
            "SELECT name_embedding FROM meme_packs WHERE id = ?1",
            [pack.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        decode_embedding(&stored_pack_name),
        FakeEmbeddingProvider::text_values("Reactions", 4)
    );
    let stored_pack_description: Vec<u8> = raw
        .query_row(
            "SELECT description_embedding FROM meme_packs WHERE id = ?1",
            [pack.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        decode_embedding(&stored_pack_description),
        FakeEmbeddingProvider::text_values("Everyday reactions", 4)
    );
    let (stored_meme_name, stored_meme_description): (Vec<u8>, Vec<u8>) = raw
        .query_row(
            "SELECT name_embedding, description_embedding FROM memes WHERE id = ?1",
            [meme.id.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        decode_embedding(&stored_meme_name),
        FakeEmbeddingProvider::text_values("Greeting", 4)
    );
    assert_eq!(
        decode_embedding(&stored_meme_description),
        FakeEmbeddingProvider::text_values("A mixed Meme", 4)
    );
    let stored_text: Vec<u8> = raw
        .query_row(
            "SELECT embedding FROM meme_contents WHERE kind = 'text' AND meme_id = ?1",
            [meme.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        decode_embedding(&stored_text),
        FakeEmbeddingProvider::text_values("hello", 4)
    );
    drop(raw);

    let updated = database
        .update_meme_metadata(
            meme.id,
            UpdateMemeMetadata {
                name: Some("Updated".to_owned()),
                description: None,
            },
        )
        .unwrap();
    let unchanged_image = match &updated.contents[1] {
        MemeContent::Image(image) => image,
        other => panic!("expected image content, got {other:?}"),
    };
    assert_eq!(unchanged_image.relative_path, image.relative_path);

    let destination = database
        .create_meme_pack(NewMemePack {
            name: "Archive".to_owned(),
            description: None,
            author: None,
            source: None,
        })
        .unwrap();
    let moved = database.move_meme(meme.id, destination.id).unwrap();
    assert_eq!(moved.meme_pack_id, destination.id);
    assert!(database.list_memes(pack.id).unwrap().is_empty());
    assert_eq!(
        database.list_memes(destination.id).unwrap(),
        vec![moved.clone()]
    );
    assert_eq!(database.list_all_memes().unwrap(), vec![moved.clone()]);

    let old_managed_path = managed_path;
    let replaced = database
        .replace_meme_contents(
            meme.id,
            vec![NewMemeContent::Text {
                text: "replacement".to_owned(),
            }],
        )
        .unwrap();
    assert_eq!(replaced.contents.len(), 1);
    assert!(matches!(
        &replaced.contents[0],
        MemeContent::Text(text) if text.text == "replacement"
    ));
    assert!(!old_managed_path.exists());

    database
        .update_meme_pack(
            destination.id,
            UpdateMemePack {
                name: "Archived".to_owned(),
                description: Some("Stored".to_owned()),
                author: None,
                source: None,
            },
        )
        .unwrap();
    assert_eq!(
        database.get_meme_pack(destination.id).unwrap().name,
        "Archived"
    );
    database.delete_meme(meme.id).unwrap();
    assert!(matches!(database.get_meme(meme.id), Err(Error::MemeNotFound(id)) if id == meme.id));
    database.delete_meme_pack(destination.id).unwrap();
    assert!(matches!(
        database.get_meme_pack(destination.id),
        Err(Error::MemePackNotFound(id)) if id == destination.id
    ));
}

#[test]
fn detects_all_supported_formats_and_embeds_the_first_gif_frame() {
    let directory = tempfile::tempdir().unwrap();
    let sources = directory.path().join("sources");
    fs::create_dir(&sources).unwrap();
    let png = sources.join("image.bin");
    let jpeg = sources.join("photo.bin");
    let webp = sources.join("web.bin");
    let gif = sources.join("animated.bin");
    write_static_image(&png, EncodedImageFormat::Png, 2, 3, [200, 0, 0, 255]);
    write_static_image(&jpeg, EncodedImageFormat::Jpeg, 4, 2, [0, 200, 0, 255]);
    write_static_image(&webp, EncodedImageFormat::WebP, 3, 4, [0, 0, 200, 255]);
    write_animated_gif(&gif);

    let mut database = MemeDatabase::open(
        directory.path().join("library"),
        FakeEmbeddingProvider::valid(),
    )
    .unwrap();
    let pack = simple_pack(&mut database, "Formats");
    let meme = database
        .create_meme(
            pack.id,
            NewMeme {
                name: None,
                description: None,
                contents: [&png, &jpeg, &webp, &gif]
                    .into_iter()
                    .map(|path| NewMemeContent::Image {
                        source_path: path.clone(),
                    })
                    .collect(),
            },
        )
        .unwrap();

    let formats = meme
        .contents
        .iter()
        .map(|content| match content {
            MemeContent::Image(image) => image.format,
            other => panic!("expected image content, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        formats,
        vec![
            ImageFormat::Png,
            ImageFormat::Jpeg,
            ImageFormat::WebP,
            ImageFormat::Gif
        ]
    );
    let managed_paths = meme
        .contents
        .iter()
        .map(|content| match content {
            MemeContent::Image(image) => database.resolve_media_path(&image.relative_path).unwrap(),
            other => panic!("expected image content, got {other:?}"),
        })
        .collect::<Vec<_>>();
    let gif_content = match &meme.contents[3] {
        MemeContent::Image(image) => image,
        other => panic!("expected GIF content, got {other:?}"),
    };
    assert_eq!((gif_content.width, gif_content.height), (2, 2));

    let raw = Connection::open(database.database_path()).unwrap();
    let gif_embedding: Vec<u8> = raw
        .query_row(
            "SELECT embedding FROM meme_contents WHERE id = ?1",
            [gif_content.id.to_string()],
            |row| row.get(0),
        )
        .unwrap();
    let first_frame = image::ImageReader::open(&gif)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    assert_eq!(
        decode_embedding(&gif_embedding),
        FakeEmbeddingProvider::image_values(&first_frame, 4)
    );
    drop(raw);
    database.delete_meme_pack(pack.id).unwrap();
    assert!(managed_paths.iter().all(|path| !path.exists()));
    assert!(matches!(
        database.get_meme(meme.id),
        Err(Error::MemeNotFound(id)) if id == meme.id
    ));
}

#[test]
fn combines_direct_and_inherited_tags_without_materializing_inheritance() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        MemeDatabase::open(directory.path(), FakeEmbeddingProvider::valid()).unwrap();
    let pack = simple_pack(&mut database, "Tagged");
    let meme = database
        .create_meme(
            pack.id,
            NewMeme {
                name: None,
                description: None,
                contents: vec![NewMemeContent::Text {
                    text: "tag me".to_owned(),
                }],
            },
        )
        .unwrap();
    let cat = database
        .create_tag(NewTag {
            name: "Cat".to_owned(),
        })
        .unwrap();
    let reaction = database
        .create_tag(NewTag {
            name: "reaction".to_owned(),
        })
        .unwrap();
    let reaction = database
        .rename_tag(reaction.id, "Emotion".to_owned())
        .unwrap();
    assert_eq!(reaction.name, "Emotion");
    assert!(matches!(
        database.rename_tag(reaction.id, "CAT".to_owned()),
        Err(Error::DuplicateTag(name)) if name == "CAT"
    ));
    assert!(matches!(
        database.create_tag(NewTag { name: " cAt ".to_owned() }),
        Err(Error::DuplicateTag(name)) if name == "cAt"
    ));

    database.attach_tag_to_meme_pack(pack.id, cat.id).unwrap();
    database.attach_tag_to_meme(meme.id, cat.id).unwrap();
    database
        .attach_tag_to_meme_pack(pack.id, reaction.id)
        .unwrap();
    assert_eq!(
        database.list_meme_effective_tags(meme.id).unwrap(),
        vec![
            EffectiveTag {
                tag: cat.clone(),
                direct: true,
                inherited: true,
            },
            EffectiveTag {
                tag: reaction.clone(),
                direct: false,
                inherited: true,
            },
        ]
    );
    assert!(matches!(
        database.attach_tag_to_meme(meme.id, cat.id),
        Err(Error::TagAssociationExists { target: "Meme", target_id, tag_id })
            if target_id == meme.id && tag_id == cat.id
    ));

    database.detach_tag_from_meme_pack(pack.id, cat.id).unwrap();
    assert_eq!(
        database.list_meme_effective_tags(meme.id).unwrap(),
        vec![
            EffectiveTag {
                tag: cat.clone(),
                direct: true,
                inherited: false,
            },
            EffectiveTag {
                tag: reaction,
                direct: false,
                inherited: true,
            },
        ]
    );
    assert!(matches!(
        database.detach_tag_from_meme_pack(pack.id, cat.id),
        Err(Error::TagAssociationNotFound { target: "MemePack", target_id, tag_id })
            if target_id == pack.id && tag_id == cat.id
    ));
    database.delete_tag(cat.id).unwrap();
    assert!(database.list_meme_direct_tags(meme.id).unwrap().is_empty());
}

#[test]
fn rejects_invalid_inputs_and_rolls_back_provider_failures() {
    let directory = tempfile::tempdir().unwrap();
    let mut database =
        MemeDatabase::open(directory.path(), FakeEmbeddingProvider::valid()).unwrap();
    assert!(matches!(
        database.create_meme_pack(NewMemePack {
            name: "   ".to_owned(),
            description: None,
            author: None,
            source: None,
        }),
        Err(Error::EmptyField {
            field: "MemePack name"
        })
    ));
    let pack = simple_pack(&mut database, "Valid");
    assert!(matches!(
        database.create_meme(
            pack.id,
            NewMeme {
                name: None,
                description: None,
                contents: vec![]
            }
        ),
        Err(Error::EmptyMemeContents)
    ));
    let unsupported = directory.path().join("not-an-image.txt");
    fs::write(&unsupported, b"not an image").unwrap();
    assert!(matches!(
        database.create_meme(
            pack.id,
            NewMeme {
                name: None,
                description: None,
                contents: vec![NewMemeContent::Image { source_path: unsupported.clone() }],
            }
        ),
        Err(Error::UnsupportedImageFormat(path)) if path == unsupported
    ));
    assert!(database.list_memes(pack.id).unwrap().is_empty());

    let wrong_directory = tempfile::tempdir().unwrap();
    let mut wrong = FakeEmbeddingProvider::valid();
    wrong.wrong_output_dimension = Some(3);
    let mut wrong_database = MemeDatabase::open(wrong_directory.path(), wrong).unwrap();
    assert!(matches!(
        wrong_database.create_meme_pack(NewMemePack {
            name: "Wrong vector".to_owned(),
            description: None,
            author: None,
            source: None,
        }),
        Err(Error::InvalidEmbeddingDimension {
            field: "MemePack name",
            expected: 4,
            actual: 3,
        })
    ));
    assert!(wrong_database.list_meme_packs().unwrap().is_empty());

    let failed_directory = tempfile::tempdir().unwrap();
    let mut failing = FakeEmbeddingProvider::valid();
    failing.fail = true;
    let mut failed_database = MemeDatabase::open(failed_directory.path(), failing).unwrap();
    assert!(matches!(
        failed_database.create_meme_pack(NewMemePack {
            name: "Failure".to_owned(),
            description: None,
            author: None,
            source: None,
        }),
        Err(Error::EmbeddingProvider(_))
    ));
    assert!(failed_database.list_meme_packs().unwrap().is_empty());

    let zero_directory = tempfile::tempdir().unwrap();
    let mut zero_database = MemeDatabase::open(
        zero_directory.path(),
        FixedVectorProvider {
            values: vec![0.0; 4],
        },
    )
    .unwrap();
    assert!(matches!(
        zero_database.create_tag(NewTag {
            name: "zero".to_owned()
        }),
        Err(Error::ZeroEmbedding { field: "Tag name" })
    ));

    let non_finite_directory = tempfile::tempdir().unwrap();
    let mut non_finite_database = MemeDatabase::open(
        non_finite_directory.path(),
        FixedVectorProvider {
            values: vec![1.0, f32::NAN, 2.0, 3.0],
        },
    )
    .unwrap();
    assert!(matches!(
        non_finite_database.create_tag(NewTag {
            name: "non-finite".to_owned()
        }),
        Err(Error::NonFiniteEmbedding {
            field: "Tag name",
            index: 1,
        })
    ));
}

#[test]
fn enforces_embedding_compatibility_schema_version_and_media_integrity() {
    let directory = tempfile::tempdir().unwrap();
    let storage = directory.path().join("library");
    {
        let mut database = MemeDatabase::open(&storage, FakeEmbeddingProvider::valid()).unwrap();
        let pack = simple_pack(&mut database, "Integrity");
        let source = directory.path().join("source.png");
        write_static_image(&source, EncodedImageFormat::Png, 2, 2, [1, 2, 3, 255]);
        let meme = database
            .create_meme(
                pack.id,
                NewMeme {
                    name: None,
                    description: None,
                    contents: vec![NewMemeContent::Image {
                        source_path: source,
                    }],
                },
            )
            .unwrap();
        let image = match &meme.contents[0] {
            MemeContent::Image(image) => image,
            other => panic!("expected image content, got {other:?}"),
        };
        fs::remove_file(database.resolve_media_path(&image.relative_path).unwrap()).unwrap();
        assert!(matches!(
            database.get_meme(meme.id),
            Err(Error::MissingMedia(_))
        ));
    }

    let mut different_model = FakeEmbeddingProvider::valid();
    different_model.model_id = "test-embedding-v2".to_owned();
    assert!(matches!(
        MemeDatabase::open(&storage, different_model),
        Err(Error::IncompatibleEmbeddingSpace {
            expected_model,
            actual_model,
            expected_dimension: 4,
            actual_dimension: 4,
        }) if expected_model == "test-embedding-v1" && actual_model == "test-embedding-v2"
    ));

    let raw = Connection::open(storage.join("memelith.sqlite3")).unwrap();
    raw.pragma_update(None, "user_version", 99).unwrap();
    drop(raw);
    assert!(matches!(
        MemeDatabase::open(&storage, FakeEmbeddingProvider::valid()),
        Err(Error::UnsupportedSchemaVersion {
            expected: 1,
            actual: 99
        })
    ));
}

#[test]
fn removes_staging_and_orphaned_media_when_reopening() {
    let directory = tempfile::tempdir().unwrap();
    let storage = directory.path().join("library");
    drop(MemeDatabase::open(&storage, FakeEmbeddingProvider::valid()).unwrap());
    let orphan = storage.join("media/images/orphan.png");
    let staged = storage.join(".staging/unfinished.tmp");
    fs::write(&orphan, b"orphan").unwrap();
    fs::write(&staged, b"unfinished").unwrap();

    drop(MemeDatabase::open(&storage, FakeEmbeddingProvider::valid()).unwrap());

    assert!(!orphan.exists());
    assert!(!staged.exists());
}

fn simple_pack(database: &mut MemeDatabase, name: &str) -> memelith_core::MemePack {
    database
        .create_meme_pack(NewMemePack {
            name: name.to_owned(),
            description: None,
            author: None,
            source: None,
        })
        .unwrap()
}

fn write_static_image(
    path: &Path,
    format: EncodedImageFormat,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    let image = DynamicImage::ImageRgba8(ImageBuffer::from_pixel(width, height, Rgba(color)));
    image.save_with_format(path, format).unwrap();
}

fn write_animated_gif(path: &Path) {
    let first: RgbaImage = ImageBuffer::from_pixel(2, 2, Rgba([240, 0, 0, 255]));
    let second: RgbaImage = ImageBuffer::from_pixel(2, 2, Rgba([0, 0, 240, 255]));
    let file = File::create(path).unwrap();
    let mut encoder = GifEncoder::new(file);
    encoder
        .encode_frames([
            Frame::from_parts(first, 0, 0, Delay::from_numer_denom_ms(100, 1)),
            Frame::from_parts(second, 0, 0, Delay::from_numer_denom_ms(100, 1)),
        ])
        .unwrap();
}

fn decode_embedding(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
}
