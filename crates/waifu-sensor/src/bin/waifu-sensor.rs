use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand, ValueEnum};
use waifu_sensor::{
    AddCharacterOptions, BuiltinAssets, Bundle, CharacterMatch, ExecutionPolicy, MlDanbooruTagger,
    ModelManager, ModelManifest, PlatformPaths, Provider, WaifuDatabase, WaifuSensor,
    image::{self, DynamicImage},
    rusqlite::Connection,
    training::{OptimizationConfig, TaggedDataset, optimize_features},
};

#[derive(Debug, Parser)]
#[command(
    name = "waifu-sensor",
    version,
    about = "Rust waifu-sensor library tools"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Db(DbCommand),
    Predict(PredictCommand),
    WhyNot(WhyNotCommand),
    Character(CharacterCommand),
    TrainFeatures(TrainFeaturesCommand),
    Paths(PathsCommand),
}

#[derive(Debug, Args)]
struct PathsCommand {}

#[derive(Debug, Args)]
struct DbCommand {
    #[command(subcommand)]
    command: DbSubcommand,
}

#[derive(Debug, Subcommand)]
enum DbSubcommand {
    Sync(DatabaseOptions),
}

#[derive(Clone, Debug, Args)]
struct DatabaseOptions {
    /// Override the platform-native SQLite database path.
    #[arg(long)]
    database: Option<PathBuf>,
    /// Override the platform or built-in character bundle.
    #[arg(long)]
    bundle: Option<PathBuf>,
}

#[derive(Clone, Debug, Args)]
struct RuntimeOptions {
    #[command(flatten)]
    database: DatabaseOptions,
    /// Override the built-in model manifest and adjacent class list.
    #[arg(long)]
    model_manifest: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ProviderArgument::Auto)]
    provider: ProviderArgument,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProviderArgument {
    Auto,
    Cpu,
    CoreMl,
    DirectMl,
    Cuda,
}

impl ProviderArgument {
    fn policy(self) -> ExecutionPolicy {
        match self {
            Self::Auto => ExecutionPolicy::Auto,
            Self::Cpu => ExecutionPolicy::Cpu,
            Self::CoreMl => ExecutionPolicy::Require(Provider::CoreMl),
            Self::DirectMl => ExecutionPolicy::Require(Provider::DirectMl),
            Self::Cuda => ExecutionPolicy::Require(Provider::Cuda),
        }
    }
}

#[derive(Debug, Args)]
struct PredictCommand {
    #[command(flatten)]
    runtime: RuntimeOptions,
    image: PathBuf,
    #[arg(long, default_value_t = 3)]
    top: usize,
}

#[derive(Debug, Args)]
struct WhyNotCommand {
    #[command(flatten)]
    runtime: RuntimeOptions,
    image: PathBuf,
    /// Character name to compare against.
    character: String,
    #[arg(long, default_value_t = 3)]
    top: usize,
}

#[derive(Debug, Args)]
struct CharacterCommand {
    #[command(subcommand)]
    command: CharacterSubcommand,
}

#[derive(Debug, Subcommand)]
enum CharacterSubcommand {
    Add {
        #[command(flatten)]
        runtime: RuntimeOptions,
        #[arg(long)]
        name: String,
        #[arg(required = true)]
        images: Vec<PathBuf>,
        #[arg(long, default_value_t = 0.25)]
        prototype_merge_distance: f32,
    },
    AddImages {
        #[command(flatten)]
        runtime: RuntimeOptions,
        /// Character name receiving the additional images.
        #[arg(long)]
        character: String,
        #[arg(required = true)]
        images: Vec<PathBuf>,
        #[arg(long, default_value_t = 0.25)]
        prototype_merge_distance: f32,
    },
}

#[derive(Debug, Args)]
struct TrainFeaturesCommand {
    #[arg(long)]
    training: PathBuf,
    #[arg(long)]
    validation: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long, default_value = "waifu-sensor-optimized-v1")]
    schema_id: String,
    #[arg(long, default_value = "ml_caformer_m36_dec-5-97527")]
    model_id: String,
    #[arg(long, default_value_t = 1024)]
    candidate_limit: usize,
    #[arg(long, default_value_t = 256)]
    iterations: usize,
    #[arg(long, default_value_t = 1)]
    seed: u64,
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Db(command) => run_db(command),
        Command::Predict(command) => run_predict(command),
        Command::WhyNot(command) => run_why_not(command),
        Command::Character(command) => run_character(command),
        Command::TrainFeatures(command) => run_train_features(command),
        Command::Paths(command) => run_paths(command),
    }
}

fn run_db(command: DbCommand) -> Result<()> {
    match command.command {
        DbSubcommand::Sync(options) => {
            let bundle = resolve_bundle(options.bundle.as_deref())?;
            let database = resolve_database(options.database.as_deref())?;
            let connection = open_database(&database)?;
            let (database, sync) = WaifuDatabase::open(connection, &bundle)?;
            println!("{sync:?}: {} characters", database.character_count()?);
            Ok(())
        }
    }
}

fn run_predict(command: PredictCommand) -> Result<()> {
    let mut sensor = open_sensor(&command.runtime)?;
    let image = open_image(&command.image)?;
    let matches = sensor.predict(&image, positive_top(command.top)?)?;
    write_prediction_text(std::io::stdout().lock(), &matches)?;
    Ok(())
}

fn write_prediction_text(mut writer: impl Write, matches: &[CharacterMatch]) -> Result<()> {
    for item in matches {
        writeln!(writer, "{}\t{}", item.distance, item.name)?;
    }
    Ok(())
}

fn run_why_not(command: WhyNotCommand) -> Result<()> {
    let mut sensor = open_sensor(&command.runtime)?;
    let image = open_image(&command.image)?;
    let character = sensor.character_id(&command.character)?;
    let differences = sensor.why_not(&image, character, positive_top(command.top)?)?;
    for item in differences {
        println!(
            "{}\t{}\tquery={}\ttarget={}",
            item.tag, item.weighted_delta, item.query_probability, item.target_probability
        );
    }
    Ok(())
}

fn run_character(command: CharacterCommand) -> Result<()> {
    match command.command {
        CharacterSubcommand::Add {
            runtime,
            name,
            images,
            prototype_merge_distance,
        } => {
            let mut sensor = open_sensor(&runtime)?;
            let images = open_images(&images)?;
            sensor.add_character(
                name.as_str(),
                &images,
                AddCharacterOptions {
                    prototype_merge_distance,
                },
            )?;
            println!("{name}");
            Ok(())
        }
        CharacterSubcommand::AddImages {
            runtime,
            character,
            images,
            prototype_merge_distance,
        } => {
            let mut sensor = open_sensor(&runtime)?;
            let images = open_images(&images)?;
            let character_id = sensor.character_id(&character)?;
            sensor.add_character_images(
                character_id,
                &images,
                AddCharacterOptions {
                    prototype_merge_distance,
                },
            )?;
            println!("{character}");
            Ok(())
        }
    }
}

fn run_train_features(command: TrainFeaturesCommand) -> Result<()> {
    let training = TaggedDataset::from_path(&command.training)?;
    let validation = TaggedDataset::from_path(&command.validation)?;
    let config = OptimizationConfig {
        schema_id: command.schema_id,
        model_id: command.model_id,
        candidate_limit: command.candidate_limit,
        iterations: command.iterations,
        seed: command.seed,
        ..OptimizationConfig::default()
    };
    let result = optimize_features(&training, &validation, &config)?;
    let writer = BufWriter::new(File::create(&command.output)?);
    serde_json::to_writer_pretty(writer, &result)?;
    eprintln!(
        "top1={:.6} top3={:.6} features={} evaluations={}",
        result.accuracy.top1,
        result.accuracy.top3,
        result.schema.features.len(),
        result.evaluations
    );
    Ok(())
}

fn run_paths(command: PathsCommand) -> Result<()> {
    let paths = PlatformPaths::discover()?;
    let PathsCommand {} = command;
    println!("data\t{}", paths.data_dir.display());
    println!("database\t{}", paths.database.display());
    println!("bundles\t{}", paths.bundles.display());
    Ok(())
}

fn open_sensor(options: &RuntimeOptions) -> Result<WaifuSensor> {
    let bundle = resolve_bundle(options.database.bundle.as_deref())?;
    let model_resources = resolve_model_resources(options.model_manifest.as_deref())?;
    let model_manifest = &model_resources.manifest;
    if model_manifest.id != bundle.feature_schema.model_id {
        bail!(
            "model `{}` does not match bundle model `{}`",
            model_manifest.id,
            bundle.feature_schema.model_id
        );
    }
    ModelManager::verify(model_manifest, &model_resources.model_path)?;
    let tagger = match model_resources.classes {
        ModelClasses::Builtin(classes) => MlDanbooruTagger::load_with_classes(
            &model_resources.model_path,
            &classes,
            bundle.feature_schema.clone(),
            options.provider.policy(),
        )?,
        ModelClasses::Path(path) => MlDanbooruTagger::load(
            &model_resources.model_path,
            path,
            bundle.feature_schema.clone(),
            options.provider.policy(),
        )?,
    };
    let database = resolve_database(options.database.database.as_deref())?;
    let connection = open_database(&database)?;
    let (sensor, _) = WaifuSensor::open(connection, &bundle, tagger)?;
    Ok(sensor)
}

fn open_image(path: &Path) -> Result<DynamicImage> {
    image::open(path).with_context(|| format!("failed to open image {}", path.display()))
}

fn open_images(paths: &[PathBuf]) -> Result<Vec<DynamicImage>> {
    paths.iter().map(|path| open_image(path)).collect()
}

fn positive_top(value: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(value).context("--top must be greater than zero")
}

fn resolve_database(override_path: Option<&Path>) -> Result<PathBuf> {
    match override_path {
        Some(path) => Ok(path.to_owned()),
        None => Ok(PlatformPaths::discover()?.database),
    }
}

#[derive(Debug, Eq, PartialEq)]
enum BundleSource {
    Directory(PathBuf),
    Builtin,
}

fn select_bundle_source(override_path: Option<&Path>, paths: &PlatformPaths) -> BundleSource {
    if let Some(path) = override_path {
        return BundleSource::Directory(path.to_owned());
    }
    let installed = paths.bundles.join("current");
    if installed.join("manifest.json").is_file() {
        BundleSource::Directory(installed)
    } else {
        BundleSource::Builtin
    }
}

fn resolve_bundle(override_path: Option<&Path>) -> Result<Bundle> {
    if let Some(path) = override_path {
        return Ok(Bundle::open(path)?);
    }
    let paths = PlatformPaths::discover()?;
    match select_bundle_source(None, &paths) {
        BundleSource::Directory(path) => Ok(Bundle::open(path)?),
        BundleSource::Builtin => Ok(BuiltinAssets::bundle()?),
    }
}

enum ModelClasses {
    Builtin(Vec<String>),
    Path(PathBuf),
}

struct ModelResources {
    manifest: ModelManifest,
    classes: ModelClasses,
    model_path: PathBuf,
}

fn resolve_model_manifest(override_path: Option<&Path>) -> Result<ModelManifest> {
    match override_path {
        Some(path) => ModelManifest::from_path(path)
            .with_context(|| format!("failed to load model manifest {}", path.display())),
        None => Ok(BuiltinAssets::model_manifest()?),
    }
}

fn resolve_model_resources(override_path: Option<&Path>) -> Result<ModelResources> {
    match override_path {
        Some(path) => {
            let manifest = resolve_model_manifest(Some(path))?;
            let directory = path
                .parent()
                .context("model manifest path has no parent directory")?;
            let classes = directory.join(&manifest.classes);
            let model_path = ModelManager::path_in(&manifest, directory);
            Ok(ModelResources {
                manifest,
                classes: ModelClasses::Path(classes),
                model_path,
            })
        }
        None => Ok(ModelResources {
            manifest: BuiltinAssets::model_manifest()?,
            classes: ModelClasses::Builtin(BuiltinAssets::model_classes()?),
            model_path: BuiltinAssets::model_path()?,
        }),
    }
}

fn open_database(path: &Path) -> Result<Connection> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        create_dir_all(parent)
            .with_context(|| format!("failed to create database directory {}", parent.display()))?;
    }
    Connection::open(path).with_context(|| format!("failed to open database {}", path.display()))
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{BundleSource, Cli, Command, ModelClasses};

    #[test]
    fn database_sync_accepts_the_builtin_bundle_default() {
        let cli = Cli::try_parse_from(["waifu-sensor", "db", "sync"]).unwrap();

        let Command::Db(command) = cli.command else {
            panic!("expected database command");
        };
        let super::DbSubcommand::Sync(options) = command.command;
        assert_eq!(options.bundle, None);
        assert_eq!(options.database, None);
    }

    #[test]
    fn predict_accepts_builtin_runtime_resources() {
        let cli = Cli::try_parse_from(["waifu-sensor", "predict", "image.png"]).unwrap();

        let Command::Predict(command) = cli.command else {
            panic!("expected predict command");
        };
        assert_eq!(command.runtime.database.bundle, None);
        assert_eq!(command.runtime.model_manifest, None);
        assert_eq!(command.image, std::path::PathBuf::from("image.png"));
    }

    #[test]
    fn why_not_accepts_a_character_name_with_spaces() {
        let cli =
            Cli::try_parse_from(["waifu-sensor", "why-not", "image.png", "tomari mari"]).unwrap();

        let Command::WhyNot(command) = cli.command else {
            panic!("expected why-not command");
        };
        assert_eq!(command.character.to_string(), "tomari mari");
    }

    #[test]
    fn why_not_rejects_the_old_character_flag() {
        assert!(
            Cli::try_parse_from([
                "waifu-sensor",
                "why-not",
                "image.png",
                "--character",
                "tomari mari",
            ])
            .is_err()
        );
    }

    #[test]
    fn cli_rejects_machine_readable_json_switches() {
        assert!(Cli::try_parse_from(["waifu-sensor", "predict", "--json", "image.png"]).is_err());
        assert!(Cli::try_parse_from(["waifu-sensor", "paths", "--json"]).is_err());
        assert!(
            Cli::try_parse_from([
                "waifu-sensor",
                "why-not",
                "--json",
                "image.png",
                "--character",
                "tomari mari",
            ])
            .is_err()
        );
    }

    #[test]
    fn add_images_accepts_a_character_name_with_spaces() {
        let cli = Cli::try_parse_from([
            "waifu-sensor",
            "character",
            "add-images",
            "--character",
            "tomari mari",
            "image.png",
        ])
        .unwrap();

        let Command::Character(command) = cli.command else {
            panic!("expected character command");
        };
        let super::CharacterSubcommand::AddImages {
            character, images, ..
        } = command.command
        else {
            panic!("expected add-images command");
        };
        assert_eq!(character, "tomari mari");
        assert_eq!(images, vec![std::path::PathBuf::from("image.png")]);
    }

    #[test]
    fn prediction_text_hides_the_internal_character_id() {
        let matches = vec![waifu_sensor::CharacterMatch {
            character_id: waifu_sensor::CharacterId::from_uuid(
                uuid::Uuid::parse_str("3eb63097-4d85-46d2-b188-4e107f76f565").unwrap(),
            ),
            name: "tomari mari".to_owned(),
            distance: 0.25,
        }];
        let mut output = Vec::new();

        super::write_prediction_text(&mut output, &matches).unwrap();

        assert_eq!(String::from_utf8(output).unwrap(), "0.25\ttomari mari\n");
    }

    #[test]
    fn bundle_selection_prefers_explicit_then_installed_then_builtin() {
        let directory = tempfile::tempdir().unwrap();
        let paths = waifu_sensor::PlatformPaths::from_data_root(directory.path().join("data"));

        assert_eq!(
            super::select_bundle_source(None, &paths),
            BundleSource::Builtin
        );

        let installed = paths.bundles.join("current");
        std::fs::create_dir_all(&installed).unwrap();
        std::fs::write(installed.join("manifest.json"), b"{}").unwrap();
        assert_eq!(
            super::select_bundle_source(None, &paths),
            BundleSource::Directory(installed)
        );

        let explicit = directory.path().join("explicit");
        assert_eq!(
            super::select_bundle_source(Some(&explicit), &paths),
            BundleSource::Directory(explicit)
        );
    }

    #[test]
    fn database_open_creates_a_missing_parent_directory() {
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("nested/data/waifu-sensor.sqlite3");

        let connection = super::open_database(&database).unwrap();
        connection
            .execute("CREATE TABLE proof(value INTEGER)", [])
            .unwrap();
        drop(connection);

        assert!(database.is_file());
    }

    #[test]
    fn custom_model_is_loaded_next_to_its_manifest() {
        let directory = tempfile::tempdir().unwrap();
        let manifest_path = directory.path().join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec(&serde_json::json!({
                "id": "test-model",
                "filename": "test.onnx",
                "url": "https://example.invalid/test.onnx",
                "sha256": "0000000000000000000000000000000000000000000000000000000000000000",
                "classes": "classes.json"
            }))
            .unwrap(),
        )
        .unwrap();

        let resources = super::resolve_model_resources(Some(&manifest_path)).unwrap();

        assert_eq!(resources.model_path, directory.path().join("test.onnx"));
        match resources.classes {
            ModelClasses::Path(path) => {
                assert_eq!(path, directory.path().join("classes.json"));
            }
            ModelClasses::Builtin(_) => panic!("custom model unexpectedly used built-in classes"),
        }
    }
}
