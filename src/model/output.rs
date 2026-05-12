use std::collections::BTreeMap;
use std::fmt::Write;

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
            Self::PlainText => Ok(Self::format_plain_text(files)),
            Self::Markdown => Ok(Self::format_markdown(files)),
            Self::Json => Self::format_json(files),
            Self::Xml => Ok(Self::format_xml(files)),
        }
    }

    fn estimate_plain_capacity(files: &[(String, String)]) -> usize {
        files.iter().map(|(p, c)| p.len() + c.len() + 4).sum()
    }

    fn estimate_markdown_capacity(files: &[(String, String)]) -> usize {
        files.iter().map(|(p, c)| p.len() + c.len() + 16).sum()
    }

    fn estimate_xml_capacity(files: &[(String, String)]) -> usize {
        let header = 60;
        let per_file: usize = files.iter().map(|(p, c)| p.len() + c.len() + 80).sum();
        header + per_file
    }

    fn format_plain_text(files: &[(String, String)]) -> String {
        let mut output = String::with_capacity(Self::estimate_plain_capacity(files));

        for (path, content) in files {
            let _ = writeln!(output, "[{}]", path);
            let _ = writeln!(output, "{}", content);
        }

        output
    }

    fn format_markdown(files: &[(String, String)]) -> String {
        let mut output = String::with_capacity(Self::estimate_markdown_capacity(files));

        for (path, content) in files {
            let _ = writeln!(output, "## {}\n", path);
            let _ = writeln!(output, "```");
            let _ = writeln!(output, "{}", content);
            let _ = writeln!(output, "```\n");
        }

        output
    }

    fn format_json(files: &[(String, String)]) -> SwarmResult<String> {
        let map: BTreeMap<&str, &str> = files.iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
            .collect();

        serde_json::to_string_pretty(&map)
            .map_err(|error| SwarmError::Other(format!("Failed to serialize JSON: {}", error)))
    }

    fn format_xml(files: &[(String, String)]) -> String {
        let mut output = String::with_capacity(Self::estimate_xml_capacity(files));

        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n");

        for (path, content) in files {
            output.push_str("  <file>\n    <path>");
            append_xml_escaped(&mut output, path);
            output.push_str("</path>\n    <content><![CDATA[");
            append_cdata_escaped(&mut output, content);
            output.push_str("]]></content>\n  </file>\n");
        }

        output.push_str("</files>\n");

        output
    }
}

fn append_xml_escaped(output_string: &mut String, s: &str) {
    for c in s.chars() {
        match c {
            '&' => output_string.push_str("&amp;"),
            '<' => output_string.push_str("&lt;"),
            '>' => output_string.push_str("&gt;"),
            '"' => output_string.push_str("&quot;"),
            '\'' => output_string.push_str("&apos;"),
            _ => output_string.push(c),
        }
    }
}

fn append_cdata_escaped(output_string: &mut String, s: &str) {
    let pattern = "]]>";
    let mut start = 0;

    while let Some(position) = s[start..].find(pattern) {
        output_string.push_str(&s[start..start + position]);
        output_string.push_str("]]]]><![CDATA[>");
        start += position + pattern.len();
    }

    output_string.push_str(&s[start..]);
}
