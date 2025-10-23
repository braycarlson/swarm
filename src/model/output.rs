use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

use serde::{Deserialize, Serialize};

use crate::model::error::{SwarmError, SwarmResult};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum OutputFormat {
    #[default]
    PlainText,
    Markdown,
    Json,
    Xml,
}

impl OutputFormat {
    pub fn name(&self) -> &str {
        match self {
            Self::PlainText => "Plain Text",
            Self::Markdown => "Markdown",
            Self::Json => "JSON",
            Self::Xml => "XML",
        }
    }

    pub fn all() -> &'static [OutputFormat] {
        &[
            Self::PlainText,
            Self::Markdown,
            Self::Json,
            Self::Xml,
        ]
    }

    pub fn format(&self, files: &[(String, String)]) -> SwarmResult<String> {
        match self {
            Self::PlainText => Self::format_plain_text(files),
            Self::Markdown => Self::format_markdown(files),
            Self::Json => Self::format_json(files),
            Self::Xml => Self::format_xml(files),
        }
    }

    fn format_plain_text(files: &[(String, String)]) -> SwarmResult<String> {
        let mut output = String::new();

        for (path, content) in files {
            writeln!(output, "[{}]", path)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "{}", content)
                .map_err(|e| SwarmError::Other(format!("Failed to write content: {}", e)))?;
        }

        Ok(output)
    }

    fn format_markdown(files: &[(String, String)]) -> SwarmResult<String> {
        let mut output = String::new();

        for (path, content) in files {
            writeln!(output, "## {}\n", path)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "```")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "{}", content)
                .map_err(|e| SwarmError::Other(format!("Failed to write content: {}", e)))?;
            writeln!(output, "```\n")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
        }

        Ok(output)
    }

    fn format_json(files: &[(String, String)]) -> SwarmResult<String> {
        let map: HashMap<&str, &str> = files.iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect();

        serde_json::to_string_pretty(&map)
            .map_err(|e| SwarmError::Other(format!("Failed to serialize JSON: {}", e)))
    }

    fn format_xml(files: &[(String, String)]) -> SwarmResult<String> {
        let mut output = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n");

        for (path, content) in files {
            let escaped_path = Self::escape_xml(path);
            let escaped_content = Self::escape_xml(content);

            writeln!(output, "  <file>")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "    <path>{}</path>", escaped_path)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "    <content><![CDATA[{}]]></content>", escaped_content)
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
            writeln!(output, "  </file>")
                .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;
        }

        writeln!(output, "</files>")
            .map_err(|e| SwarmError::Other(format!("Failed to write: {}", e)))?;

        Ok(output)
    }

    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}
