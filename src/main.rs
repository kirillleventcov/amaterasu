use amaterasu::{config, Amaterasu, AmaterasuConfig, WipeMode};
use clap::{Arg, Command};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = Command::new("amaterasu")
        .version("0.1.0")
        .about("A modern, fast file secure deletion tool for Linux")
        .arg(
            Arg::new("files")
                .help("Files to securely delete")
                .num_args(1..)
                .required_unless_present("config")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("mode")
                .long("mode")
                .short('m')
                .help("Wiping mode")
                .value_parser(["fast", "standard", "paranoid"])
                .default_value("standard"),
        )
        .arg(
            Arg::new("verify")
                .long("verify")
                .short('v')
                .help("Verify wipe completion")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-progress")
                .long("no-progress")
                .help("Disable progress bar")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recursive")
                .long("recursive")
                .short('r')
                .help("Recursively delete directories and their contents")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .short('f')
                .help("Force deletion without prompts, ignore non-existent files")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .long("config")
                .help("Create default config file and exit")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-metadata-wipe")
                .long("no-metadata-wipe")
                .help("Skip metadata wiping (faster but less secure)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("metadata-passes")
                .long("metadata-passes")
                .help("Number of metadata wiping passes")
                .value_parser(clap::value_parser!(usize))
                .default_value("3"),
        )
        .get_matches();

    // Handle config creation request
    if matches.get_flag("config") {
        let config_path = config::get_config_path()?;
        config::create_default_config(&config_path)?;
        println!(
            "✅ Default configuration created at: {}",
            config_path.display()
        );
        return Ok(());
    }

    // Load configuration file after parsing CLI args
    let config_file = config::load_config().unwrap_or_else(|e| {
        eprintln!("Warning: Could not load config file: {}", e);
        eprintln!("Using default configuration");
        config::ConfigFile::default()
    });

    let input_paths: Vec<PathBuf> = matches
        .get_many::<PathBuf>("files")
        .unwrap_or_default()
        .cloned()
        .collect();

    let recursive = matches.get_flag("recursive");

    let mode = match matches.get_one::<String>("mode").unwrap().as_str() {
        "fast" => WipeMode::Fast,
        "standard" => WipeMode::Standard,
        "paranoid" => WipeMode::Paranoid,
        _ => unreachable!(),
    };

    let config = AmaterasuConfig {
        verify: matches.get_flag("verify") || config_file.defaults.verify,
        progress: (!matches.get_flag("no-progress")) && config_file.defaults.progress,
        force: matches.get_flag("force"),
        mode,
        wipe_metadata: !matches.get_flag("no-metadata-wipe"),
        metadata_passes: *matches.get_one::<usize>("metadata-passes").unwrap(),
    };

    println!("🔥 Amaterasu - Secure File Deletion");
    println!("Mode: {:?}", config.mode);

    let amaterasu = Amaterasu::new(config);

    // Collect all files to wipe (expand directories if recursive flag is set)
    let files_to_wipe = amaterasu.collect_files(&input_paths, recursive).await?;

    println!("Files to wipe: {}", files_to_wipe.len());

    amaterasu.wipe_files(&files_to_wipe).await?;
    Ok(())
}
