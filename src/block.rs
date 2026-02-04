use std::fmt::{Display, Formatter, Result};

use serde_json::{Value, json};

use crate::Block::{Divider, Header, Section};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Block {
    Header(String),
    Divider,
    Section(String),
}

impl From<Block> for Value {
    fn from(block: Block) -> Self {
        match block {
            Header(text) => json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": text,
                    "emoji": true,
                },
            }),
            Divider => json!({
                "type": "divider",
            }),
            Section(text) => json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": text,
                }
            }),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Header(text) => write!(f, "Header: {text}"),
            Divider => write!(f, "----------"),
            Section(text) => write!(f, "Section: {text}"),
        }
    }
}
