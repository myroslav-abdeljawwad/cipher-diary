use std::fs;
use std::path::Path;

use cipher_diary::{errors::DiaryError, summary};

/// Test the `summary` module of the cipher‑diary project.
/// Author: Myroslav Mokhammad Abdeljawwad
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::tempdir;

    /// Helper to read a file into a vector of trimmed lines.
    fn read_lines<P>(path: P) -> Result<Vec<String>, DiaryError>
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path)?;
        Ok(content.lines().map(|l| l.trim_end().to_owned()).collect())
    }

    /// Test that a normal journal file produces the expected summary.
    #[test]
    fn test_summary_basic() -> Result<(), DiaryError> {
        // Load example journal from data/example.journal
        let lines = read_lines("data/example.journal")?;

        // The summary should be non‑empty and contain certain keywords.
        let output = summary::summarize(&lines)?;
        assert!(!output.is_empty(), "Summary should not be empty");

        // The template is expected to wrap the highlights in a specific marker
        // e.g., "[Highlights] ... [/Highlights]"
        assert!(
            output.contains("[Highlights]") && output.contains("[/Highlights]"),
            "Output should contain highlight markers"
        );

        // Ensure that each original line appears at least once in the summary,
        // as the summarizer simply concatenates the first words.
        for line in &lines {
            if !line.is_empty() {
                assert!(
                    output.contains(line.split_whitespace().next().unwrap()),
                    "Output should contain keyword from line: {}",
                    line
                );
            }
        }

        Ok(())
    }

    /// Test that an empty journal file results in a graceful summary.
    #[test]
    fn test_summary_empty() -> Result<(), DiaryError> {
        let lines: Vec<String> = Vec::new();
        let output = summary::summarize(&lines)?;
        assert!(
            output.is_empty(),
            "Summary of empty input should be an empty string"
        );
        Ok(())
    }

    /// Test that the summarizer correctly handles lines with only whitespace.
    #[test]
    fn test_summary_whitespace_lines() -> Result<(), DiaryError> {
        let lines = vec![
            String::from("   "),
            String::from("\t"),
            String::from("First meaningful line."),
        ];
        let output = summary::summarize(&lines)?;
        assert!(
            !output.is_empty(),
            "Summary should include the non‑whitespace line"
        );
        assert!(
            output.contains("First"),
            "Output should contain keyword from non‑whitespace line"
        );
        Ok(())
    }

    /// Test that the summarizer respects a maximum number of highlights
    /// defined in the configuration.
    #[test]
    fn test_summary_max_highlights() -> Result<(), DiaryError> {
        // Create a temporary config with max 3 highlights
        let dir = tempdir()?;
        let cfg_path = dir.path().join("config.toml");
        fs::write(
            &cfg_path,
            r#"
                [summary]
                max_highlights = 3
            "#,
        )?;

        // Load configuration (assuming config module exposes a load function)
        use cipher_diary::config;
        let cfg = config::load_config(&cfg_path)?;

        // Prepare many lines
        let mut lines = Vec::new();
        for i in 1..=10 {
            lines.push(format!("Entry number {}", i));
        }

        // Summarize with the loaded configuration
        let output = summary::summarize_with_cfg(&lines, &cfg)?;
        // Count how many highlight markers appear
        let count = output.matches("[Highlights]").count();
        assert_eq!(count, cfg.summary.max_highlights as usize);

        Ok(())
    }

    /// Test that the summarizer correctly handles very long lines without truncation.
    #[test]
    fn test_summary_long_lines() -> Result<(), DiaryError> {
        let long_line = "A".repeat(10_000);
        let lines = vec![long_line.clone()];
        let output = summary::summarize(&lines)?;
        // The output should contain the long line (or its start)
        assert!(
            output.contains("AAAA"),
            "Output should preserve long content"
        );
        Ok(())
    }

    /// Test that the template rendering is deterministic.
    #[test]
    fn test_template_determinism() -> Result<(), DiaryError> {
        let lines = vec![
            String::from("First entry."),
            String::from("Second entry."),
        ];

        // Render twice
        let first_render = summary::summarize(&lines)?;
        let second_render = summary::summarize(&lines)?;

        assert_eq!(
            first_render, second_render,
            "Template rendering should be deterministic"
        );
        Ok(())
    }

    /// Test that the summarizer handles Unicode characters gracefully.
    #[test]
    fn test_summary_unicode() -> Result<(), DiaryError> {
        let lines = vec![
            String::from("Привет мир"),
            String::from("こんにちは世界"),
            String::from("¡Hola mundo!"),
        ];
        let output = summary::summarize(&lines)?;
        for line in &lines {
            assert!(
                output.contains(line.split_whitespace().next().unwrap()),
                "Output should contain Unicode keyword from line: {}",
                line
            );
        }
        Ok(())
    }

    /// Test that the summarizer returns an error when the template file is missing.
    #[test]
    fn test_summary_missing_template() {
        // Temporarily rename the template to simulate missing file
        let tmpl_path = Path::new("templates/summaries.tmpl");
        let tmp_backup = tmpl_path.with_extension("tmpl.bak");
        fs::rename(tmpl_path, &tmp_backup).expect("Could not backup template");

        // Ensure cleanup after test
        struct RestoreTemplate(PathBuf);
        impl Drop for RestoreTemplate {
            fn drop(&mut self) {
                if self.0.exists() {
                    let _ = fs::remove_file(self.0.clone());
                }
                let tmpl_path = Path::new("templates/summaries.tmpl");
                let backup = tmpl_path.with_extension("tmpl.bak");
                let _ = fs::rename(backup, tmpl_path);
            }
        }
        let _restore = RestoreTemplate(tmp_backup);

        // Attempt to summarize
        let lines = vec![String::from("Test line")];
        let result = summary::summarize(&lines);
        assert!(
            matches!(result, Err(DiaryError::TemplateNotFound(_))),
            "Expected TemplateNotFound error when template is missing"
        );
    }
}