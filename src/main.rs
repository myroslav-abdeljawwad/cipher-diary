use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use clap::Parser;

use crate::{
    cli::Cli,
    config::Config,
    encryptor::encrypt_line,
    errors::AppError,
    summary::generate_summary,
};

/// cipher-diary
///
/// Encrypt every line of your journal and let an AI whisper the highlights—no server, just local privacy.
/// Version 0.1.0 by Myroslav Mokhammad Abdeljawwad
#[derive(Parser)]
#[command(name = "cipher-diary")]
#[command(author = "Myroslav Mokhammad Abdeljawwad <myroslav@example.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Encrypt journal entries and generate AI‑powered summaries.")]
struct Args {
    /// Path to the plain text journal file
    #[arg(short, long, value_name = "FILE")]
    input: PathBuf,

    /// Write encrypted output to this file; defaults to stdout
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Generate a summary of the journal using AI and write it to <output>.summary if --output is set,
    /// otherwise print to stderr.
    #[arg(short, long)]
    summarize: bool,
}

fn main() -> Result<(), AppError> {
    let args = Args::parse();

    // Load configuration; fall back to defaults
    let config = Config::load().unwrap_or_default();

    // Read the input file line by line
    let input_file = File::open(&args.input).map_err(|e| {
        AppError::Io(format!("Failed to open input file {}: {}", args.input.display(), e))
    })?;
    let reader = BufReader::new(input_file);

    // Prepare output destination
    let mut out_writer: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(File::create(path).map_err(|e| {
            AppError::Io(format!("Failed to create output file {}: {}", path.display(), e))
        })?),
        None => Box::new(std::io::stdout()),
    };

    // Collect lines for optional summary
    let mut plain_lines = Vec::new();
    let mut encrypted_lines = Vec::new();

    for line_result in reader.lines() {
        let line = line_result.map_err(|e| AppError::Io(format!("Read error: {}", e)))?;
        plain_lines.push(line.clone());
        let enc_line = encrypt_line(&line, &config)?;
        encrypted_lines.push(enc_line);
    }

    // Write encrypted lines
    for enc in encrypted_lines {
        writeln!(out_writer, "{}", enc).map_err(|e| AppError::Io(format!("Write error: {}", e)))?;
    }
    out_writer.flush().map_err(|e| AppError::Io(format!("Flush error: {}", e)))?;

    // If requested, generate a summary
    if args.summarize {
        let summary = generate_summary(&plain_lines)?;
        match &args.output {
            Some(out_path) => {
                let mut summary_file =
                    File::create(out_path.with_extension("summary")).map_err(|e| {
                        AppError::Io(format!(
                            "Failed to create summary file {}: {}",
                            out_path.display(),
                            e
                        ))
                    })?;
                writeln!(summary_file, "{}", summary).map_err(|e| {
                    AppError::Io(format!("Write error in summary file: {}", e))
                })?;
            }
            None => {
                eprintln!("\n--- Summary ---\n{}\n", summary);
            }
        }
    }

    Ok(())
}