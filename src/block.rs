use std::error::Error;

use serde_json::{json, Value};

#[derive(Debug)]
pub(crate) enum Block {
    Header(String),
    Divider,
    Section(String),
}

impl TryFrom<Block> for Value {
    type Error = Box<dyn Error>;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        Ok(match block {
            Block::Header(text) => json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": text,
                    "emoji": true,
                },
            }),
            Block::Divider => json!({
                "type": "divider",
            }),
            Block::Section(text) => json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": text,
                }
            }),
        })
    }
}
