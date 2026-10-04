use serde::ser::{SerializeStruct as _, Serializer};
use serde::{Deserialize, Serialize};

use crate::model::error::{SwarmError, SwarmResult};

const CDATA_TERMINATOR: &str = "]]>";
const CDATA_TERMINATOR_ESCAPED: &str = "]]]]><![CDATA[>";
const MARKDOWN_ENTRY_OVERHEAD_BYTES: usize = 15;
const PLAIN_ENTRY_OVERHEAD_BYTES: usize = 4;
const XML_ENTRY_OVERHEAD_BYTES: usize = 73;
const XML_FOOTER: &str = "</files>\n";
const XML_HEADER: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<files>\n";

pub struct OutputEntry {
    pub content: String,
    pub label: String,
}

impl Serialize for OutputEntry {
    fn serialize<Target>(&self, serializer: Target) -> Result<Target::Ok, Target::Error>
    where
        Target: Serializer,
    {
        let mut state = serializer.serialize_struct("OutputEntry", 2)?;

        state.serialize_field("path", &self.label)?;
        state.serialize_field("content", &self.content)?;

        state.end()
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum OutputFormat {
    Json,
    Markdown,
    #[default]
    PlainText,
    Xml,
}

impl OutputFormat {
    fn format_json(entries: &[OutputEntry]) -> SwarmResult<String> {
        let output = serde_json::to_string_pretty(entries)
            .map_err(|error| SwarmError::Other(format!("Failed to serialize JSON: {error}")))?;

        debug_assert!(output.starts_with('['));

        Ok(output)
    }

    fn format_markdown(entries: &[OutputEntry]) -> String {
        let capacity = capacity_for(entries, MARKDOWN_ENTRY_OVERHEAD_BYTES);
        let mut output = String::with_capacity(capacity);

        for entry in entries {
            output.push_str("## ");
            output.push_str(&entry.label);
            output.push_str("\n\n```\n");
            output.push_str(&entry.content);
            output.push_str("\n```\n\n");
        }

        assert_eq!(output.len(), capacity);
        debug_assert_eq!(output.is_empty(), entries.is_empty());

        output
    }

    fn format_plain_text(entries: &[OutputEntry]) -> String {
        let capacity = capacity_for(entries, PLAIN_ENTRY_OVERHEAD_BYTES);
        let mut output = String::with_capacity(capacity);

        for entry in entries {
            output.push('[');
            output.push_str(&entry.label);
            output.push_str("]\n");
            output.push_str(&entry.content);
            output.push('\n');
        }

        assert_eq!(output.len(), capacity);
        debug_assert_eq!(output.is_empty(), entries.is_empty());

        output
    }

    fn format_xml(entries: &[OutputEntry]) -> String {
        let capacity =
            capacity_for(entries, XML_ENTRY_OVERHEAD_BYTES) + XML_HEADER.len() + XML_FOOTER.len();

        let mut output = String::with_capacity(capacity);

        output.push_str(XML_HEADER);

        for entry in entries {
            output.push_str("  <file>\n    <path>");
            append_xml_escaped(&mut output, &entry.label);
            output.push_str("</path>\n    <content><![CDATA[");
            append_cdata_escaped(&mut output, &entry.content);
            output.push_str("]]></content>\n  </file>\n");
        }

        output.push_str(XML_FOOTER);

        assert!(output.len() >= capacity);
        debug_assert!(output.ends_with(XML_FOOTER));

        output
    }

    pub fn all() -> &'static [Self] {
        &[Self::PlainText, Self::Markdown, Self::Json, Self::Xml]
    }

    pub fn format(self, entries: &[OutputEntry]) -> SwarmResult<String> {
        match self {
            Self::Json => Self::format_json(entries),
            Self::Markdown => Ok(Self::format_markdown(entries)),
            Self::PlainText => Ok(Self::format_plain_text(entries)),
            Self::Xml => Ok(Self::format_xml(entries)),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Markdown => "Markdown",
            Self::PlainText => "Plain Text",
            Self::Xml => "XML",
        }
    }
}

fn append_cdata_escaped(output: &mut String, text: &str) {
    let output_length_before = output.len();

    for (index, part) in text.split(CDATA_TERMINATOR).enumerate() {
        if index > 0 {
            output.push_str(CDATA_TERMINATOR_ESCAPED);
        }

        output.push_str(part);
    }

    debug_assert!(output.len() >= output_length_before + text.len());
}

fn append_xml_escaped(output: &mut String, text: &str) {
    let output_length_before = output.len();

    for character in text.chars() {
        match character {
            '"' => output.push_str("&quot;"),
            '&' => output.push_str("&amp;"),
            '\'' => output.push_str("&apos;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(character),
        }
    }

    debug_assert!(output.len() >= output_length_before + text.len());
}

fn capacity_for(entries: &[OutputEntry], overhead_bytes: usize) -> usize {
    let capacity: usize = entries
        .iter()
        .map(|entry| entry.label.len() + entry.content.len() + overhead_bytes)
        .sum();

    assert!(capacity >= entries.len() * overhead_bytes);

    capacity
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<OutputEntry> {
        vec![
            OutputEntry {
                content: "beta".to_owned(),
                label: "b.rs".to_owned(),
            },
            OutputEntry {
                content: "alpha".to_owned(),
                label: "a.rs".to_owned(),
            },
            OutputEntry {
                content: "beta duplicate".to_owned(),
                label: "b.rs".to_owned(),
            },
        ]
    }

    #[test]
    fn json_preserves_order_and_duplicates() {
        let output = OutputFormat::Json
            .format(&sample())
            .expect("the entries serialize");

        let parsed: serde_json::Value = serde_json::from_str(&output).expect("the output parses");
        let array = parsed.as_array().expect("the output is an array");

        assert_eq!(array.len(), 3);
        assert_eq!(array[0]["path"], "b.rs");
        assert_eq!(array[0]["content"], "beta");
        assert_eq!(array[1]["path"], "a.rs");
        assert_eq!(array[2]["content"], "beta duplicate");
    }

    #[test]
    fn json_writes_the_path_before_the_content() {
        let output = OutputFormat::Json
            .format(&sample())
            .expect("the entries serialize");

        let path = output.find("\"path\"").expect("the path key is present");

        let content = output
            .find("\"content\"")
            .expect("the content key is present");

        assert!(path < content);
    }

    #[test]
    fn plain_text_preserves_order_and_duplicates() {
        let output = OutputFormat::PlainText
            .format(&sample())
            .expect("the entries format");

        assert!(output.starts_with("[b.rs]\nbeta\n"));
        assert_eq!(output.matches("[b.rs]").count(), 2);
    }

    #[test]
    fn markdown_preserves_order_and_duplicates() {
        let output = OutputFormat::Markdown
            .format(&sample())
            .expect("the entries format");

        assert!(output.starts_with("## b.rs\n"));
        assert_eq!(output.matches("## b.rs").count(), 2);
    }

    #[test]
    fn xml_preserves_order_and_duplicates() {
        let output = OutputFormat::Xml
            .format(&sample())
            .expect("the entries format");

        let second = output
            .find("<path>b.rs</path>")
            .expect("the first entry is present");

        let first = output
            .find("<path>a.rs</path>")
            .expect("the second entry is present");

        assert_eq!(output.matches("<path>b.rs</path>").count(), 2);
        assert!(second < first);
    }

    #[test]
    fn all_formats_accept_an_empty_entry_list() {
        for format in OutputFormat::all() {
            let output = format.format(&[]).expect("an empty list formats");

            match format {
                OutputFormat::Json => assert_eq!(output.trim(), "[]"),
                OutputFormat::Markdown | OutputFormat::PlainText => assert_eq!(output, ""),
                OutputFormat::Xml => assert!(output.ends_with("<files>\n</files>\n")),
            }
        }
    }

    #[test]
    fn xml_escapes_markup_in_paths() {
        let output = OutputFormat::Xml
            .format(&[OutputEntry {
                content: String::new(),
                label: "<a&b>\"'".to_owned(),
            }])
            .expect("the entry formats");

        assert!(output.contains("<path>&lt;a&amp;b&gt;&quot;&apos;</path>"));
    }

    #[test]
    fn cdata_escaping_splits_terminators() {
        let mut output = String::new();

        append_cdata_escaped(&mut output, "a]]>b");

        assert_eq!(output, "a]]]]><![CDATA[>b");
    }

    #[test]
    fn cdata_escaping_handles_consecutive_terminators() {
        let mut output = String::new();

        append_cdata_escaped(&mut output, "]]>]]>");

        assert_eq!(output, "]]]]><![CDATA[>]]]]><![CDATA[>");
    }

    #[test]
    fn cdata_escaping_passes_through_plain_text() {
        let mut output = String::new();

        append_cdata_escaped(&mut output, "no terminator here");

        assert_eq!(output, "no terminator here");
    }
}
