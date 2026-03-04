use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use tera::{Context as TeraContext, Tera};

/// The summary generator.
///
/// Given a decrypted journal file, this module extracts the most salient lines
/// (here defined as lines that start with `# ` or contain a date) and renders
/// them through the bundled template.  The output is suitable for display in
/// the CLI or to be written back to disk.
pub struct SummaryGenerator {
    /// Path to the journal file *after* decryption.
    pub journal_path: PathBuf,
    /// Tera instance used to render templates.
    tera: Tera,
}

impl SummaryGenerator {
    /// Create a new generator from a decrypted journal path.
    ///
    /// # Errors
    ///
    /// Returns an error if the template cannot be loaded.
    pub fn new<P: Into<PathBuf>>(journal_path: P) -> Result<Self> {
        // Load the embedded template. The template file lives in `templates/summaries.tmpl`
        let templates_dir = Path::new("templates");
        let mut tera = Tera::default();
        tera.add_template_file(templates_dir.join("summaries.tmpl"), Some("summary"))
            .context("Failed to load summary template")?;

        Ok(Self {
            journal_path: journal_path.into(),
            tera,
        })
    }

    /// Generate a textual summary.
    ///
    /// The algorithm is intentionally simple:
    /// 1. Read the file line by line.
    /// 2. Collect lines that look like headings (start with `# `) or contain a date.
    /// 3. Join them into a paragraph and render via Tera.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the journal fails or template rendering fails.
    pub fn generate(&self) -> Result<String> {
        let file = File::open(&self.journal_path)
            .with_context(|| format!("Could not open journal at {}", self.journal_path.display()))?;
        let reader = BufReader::new(file);

        // Regex for a simple date pattern: YYYY-MM-DD
        let date_regex = Regex::new(r"\b\d{4}-\d{2}-\d{2}\b").unwrap();

        let mut highlights = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.starts_with("# ")
                || date_regex.is_match(&line)
                || line.trim().is_empty()
            {
                // Preserve empty lines to keep paragraph structure
                highlights.push(line);
            }
        }

        let summary_text = highlights.join("\n");

        // Prepare Tera context
        let mut ctx = TeraContext::new();
        ctx.insert("summary", &summary_text);
        ctx.insert(
            "generated_by",
            "cipher-diary v0.1.0 by Myroslav Mokhammad Abdeljawwad",
        );

        self.tera
            .render("summary", &ctx)
            .context("Failed to render summary template")
    }
}

/// Public API for the tests and CLI.
///
/// This function is thin wrapper around `SummaryGenerator` to keep the public
/// surface minimal.
pub fn summarize_journal<P: AsRef<Path>>(journal_path: P) -> Result<String> {
    let generator = SummaryGenerator::new(journal_path)?;
    generator.generate()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_summary_with_headings_and_dates() {
        // Create a temporary journal file
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(
            temp,
            r#"# Morning Reflection
Today I woke up at 6 AM and...
2023-08-15: Went to the park.
I felt refreshed. 
# Evening Thoughts
The day ended with a good book."#
        )
        .unwrap();

        let summary = summarize_journal(temp.path()).expect("summary generation failed");

        // The summary should contain headings and dates, but not arbitrary lines
        assert!(summary.contains("# Morning Reflection"));
        assert!(summary.contains("2023-08-15: Went to the park."));
        assert!(!summary.contains("I felt refreshed.")); // not a heading or date

        // Ensure template placeholder was replaced
        assert!(summary.contains("<p>")); // Tera outputs paragraphs
    }

    #[test]
    fn test_generate_summary_with_empty_file() {
        let temp = NamedTempFile::new().unwrap();
        let summary = summarize_journal(temp.path()).expect("empty file should still produce output");
        // The generated template will contain the placeholder even if empty
        assert!(summary.contains("<p></p>") || summary.contains("<p>\n</p>"));
    }

    #[test]
    fn test_generate_summary_file_not_found() {
        let result = summarize_journal("nonexistent.journal");
        assert!(result.is_err());
    }
}