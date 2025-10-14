use serde::{Deserialize, Serialize};

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
}
