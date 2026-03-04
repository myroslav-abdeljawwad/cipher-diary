/// Cipher Diary CLI
///
/// This module provides a command‑line interface for the **cipher-diary** application.
/// It supports encrypting journal entries line‑by‑line and generating an AI‑derived
/// summary using a local template.
///
/// Author: Myroslav Mokhammad Abdeljawwad

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser, Subcommand};

use crate::config::Config;
use crate::encryptor::encrypt_line;
use crate::summary::summarize;
use crate::errors::DiaryError;

/// Top‑level command line arguments
#[derive(Parser, Debug)]
#[clap(
    name = "cipher-diary",
    version = "0.1.0",
    author = "Myroslav Mokhammad Abdeljawwad",
    about = "Encrypt journal entries and generate local AI summaries"
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Encrypt a plain‑text journal file and output an encrypted version
    Encrypt {
        /// Path to the input journal file (plain text)
        #[clap(short, long, value_parser)]
        input: PathBuf,

        /// Path to write the encrypted output. If omitted, writes to <input>.enc
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,

        /// Optional encryption key; if not supplied, a default is used from config
        #[clap(long, value_parser)]
        key: Option<String>,
    },

    /// Generate a summary of an encrypted journal file using the AI model
    Summarize {
        /// Path to the encrypted journal file
        #[clap(short, long, value_parser)]
        input: PathBuf,

        /// Output file for the plain‑text summary; defaults to <input>.summary.txt
        #[clap(short, long, value_parser)]
        output: Option<PathBuf>,

        /// Path to a Mustache template used for formatting the summary
        #[clap(long, value_parser, default_value = "templates/summaries.tmpl")]
        template: PathBuf,
    },

    /// Print configuration defaults
    Config {
        /// Show the current configuration in JSON format
        #[clap(short, long)]
        json: bool,
    },
}

impl Cli {
    /// Execute the selected subcommand.
    pub fn run(self) -> Result<(), DiaryError> {
        let config = Config::load()?;
        match self.command {
            Commands::Encrypt { input, output, key } => {
                encrypt_command(&config, &input, output.as_ref(), key.as_deref())
            }
            Commands::Summarize { input, output, template } => {
                summarize_command(&config, &input, output.as_ref(), &template)
            }
            Commands::Config { json } => {
                if json {
                    println!("{}", serde_json::to_string_pretty(&config).unwrap());
                } else {
                    println!("{:#?}", config);
                }
                Ok(())
            }
        }
    }
}

/// Encrypts a plain‑text journal file line by line.
fn encrypt_command(
    cfg: &Config,
    input_path: &Path,
    output_opt: Option<&Path>,
    key_override: Option<&str>,
) -> Result<(), DiaryError> {
    let key = key_override.unwrap_or_else(|| cfg.encryption_key.as_str());
    if key.is_empty() {
        return Err(DiaryError::MissingKey);
    }

    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);

    let out_path = output_opt
        .map(PathBuf::from)
        .unwrap_or_else(|| input_path.with_extension("enc"));

    let mut out_file = File::create(&out_path)?;

    for line in reader.lines() {
        let plain = line?;
        let cipher = encrypt_line(&plain, key)?;
        writeln!(out_file, "{}", cipher)?;
    }

    println!(
        "Encrypted {} to {}",
        input_path.display(),
        out_path.display()
    );
    Ok(())
}

/// Generates a summary from an encrypted journal file.
fn summarize_command(
    cfg: &Config,
    input_path: &Path,
    output_opt: Option<&Path>,
    template_path: &Path,
) -> Result<(), DiaryError> {
    let content = fs::read_to_string(input_path)?;
    // Decrypt each line first
    let decrypted_lines: Vec<String> = content
        .lines()
        .map(|c| encryptor::decrypt_line(c, cfg.encryption_key.as_str()))
        .collect::<Result<_, _>>()?;

    let summary_text = summarize(&decrypted_lines)?;

    // Render with template if available
    let rendered = if template_path.exists() {
        let tmpl_content = fs::read_to_string(template_path)?;
        mustache::compile_template(tmpl_content, "summary")?
            .render(&serde_json::json!({ "summary": summary_text }))?
    } else {
        summary_text.clone()
    };

    let out_path = output_opt
        .map(PathBuf::from)
        .unwrap_or_else(|| input_path.with_extension("summary.txt"));

    fs::write(&out_path, rendered)?;
    println!(
        "Summary written to {}",
        out_path.as_ref().display()
    );
    Ok(())
}

pub fn main() -> Result<(), DiaryError> {
    let cli = Cli::parse();
    cli.run()
}