use std::error::Error;

#[derive(Debug, Eq, PartialEq)]
pub enum Block {
    Header(String),
    Divider,
    Section(String),
}

impl TryFrom<Block> for serde_json::Value {
    type Error = Box<dyn Error>;

    fn try_from(block: Block) -> Result<Self, Self::Error> {
        Ok(match block {
            Block::Header(text) => serde_json::json!({
                "type": "header",
                "text": {
                    "type": "plain_text",
                    "text": text,
                    "emoji": true,
                },
            }),
            Block::Divider => serde_json::json!({
                "type": "divider",
            }),
            Block::Section(text) => serde_json::json!({
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": text,
                }
            }),
        })
    }
}
