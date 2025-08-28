use amaterasu::{Amaterasu, AmaterasuConfig, WipeMode};
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
                .required(true)
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
        .get_matches();

    let files: Vec<PathBuf> = matches
        .get_many::<PathBuf>("files")
        .unwrap()
        .cloned()
        .collect();

    let mode = match matches.get_one::<String>("mode").unwrap().as_str() {
        "fast" => WipeMode::Fast,
        "standard" => WipeMode::Standard,
        "paranoid" => WipeMode::Paranoid,
        _ => unreachable!(),
    };

    let config = AmaterasuConfig {
        verify: matches.get_flag("verify"),
        progress: !matches.get_flag("no-progress"),
        mode,
    };

    println!("🔥 Amaterasu - Secure File Deletion");
    println!("Mode: {:?}", config.mode);
    println!("Files to wipe: {}", files.len());

    let amaterasu = Amaterasu::new(config);
    amaterasu.wipe_files(&files).await?;

    println!("✅ Secure deletion completed");
    Ok(())
}
