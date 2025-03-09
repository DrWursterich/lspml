use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    path::PathBuf,
};

use anyhow::Result;
use serde_json::json;

use colored::Colorize;

use crate::command::diagnostic::{Diagnostic, Severity};

#[derive(Clone, Debug, clap::ValueEnum)]
pub enum Format {
    TEXT,
    GITLAB,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return match self {
            Format::TEXT => write!(f, "text"),
            Format::GITLAB => write!(f, "gitbal"),
        };
    }
}

pub fn print(diagnostics: HashMap<PathBuf, Vec<Diagnostic>>, format: Format) -> Result<()> {
    return match format {
        Format::TEXT => print_text(diagnostics),
        Format::GITLAB => print_code_quality(diagnostics),
    };
}

fn print_text(diagnostics: HashMap<PathBuf, Vec<Diagnostic>>) -> Result<()> {
    for (file, values) in diagnostics {
        let file = file
            .strip_prefix(std::env::current_dir()?)
            .map_err(|_| anyhow::anyhow!("cannot analyze files outside of working directory"))?
            .to_string_lossy()
            .to_string();
        println!("{}", file.underline());
        for diagnostic in values {
            let severity = format!(
                "{}{}{}",
                "[".blue(),
                match diagnostic.severity {
                    Severity::Hint => "HINT".green(),
                    // Severity::Information => "INFO".blue(),
                    Severity::Warning => "WARNING".yellow(),
                    Severity::Error => "ERROR".red(),
                    Severity::Critical => "CRITICAL".red(),
                },
                "]".blue()
            );
            println!(
                "{:<10} {} ({}) on line {}",
                severity.bold(),
                diagnostic.message,
                diagnostic.r#type.to_string().bold(),
                diagnostic.range.start.line + 1,
            );
        }
        println!();
    }
    return Ok(());
}

fn print_code_quality(diagnostics: HashMap<PathBuf, Vec<Diagnostic>>) -> Result<()> {
    let mut array: Vec<serde_json::Value> = Vec::new();
    for (file, values) in diagnostics {
        for diagnostic in values {
            let file = file
                .strip_prefix(std::env::current_dir()?)
                .map_err(|_| anyhow::anyhow!("cannot analyze files outside of working directory"))?
                .to_string_lossy()
                .to_string();
            let severity = match diagnostic.severity {
                Severity::Hint => continue,
                // Severity::Information => "info",
                Severity::Warning => "minor",
                Severity::Error => "major",
                Severity::Critical => "critical",
            };
            array.push(json!({
                "description": diagnostic.message,
                "check_name": diagnostic.r#type.to_string(),
                "fingerprint": diagnostic.fingerprint,
                "severity": severity,
                "location": {
                    "path": file,
                    "lines": {
                        "begin": diagnostic.range.start.line + 1,
                    }
                }
            }));
        }
    }
    println!("{}", serde_json::to_string(&array)?);
    return Ok(());
}
