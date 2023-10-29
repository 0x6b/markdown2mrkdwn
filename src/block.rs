#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Block {
    Header(String),
    Divider,
    Section(String),
}

impl From<Block> for serde_json::Value {
    fn from(block: Block) -> Self {
        match block {
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
        }
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::Header(text) => write!(f, "Header: {text}"),
            Block::Divider => write!(f, "----------"),
            Block::Section(text) => write!(f, "Section: {text}"),
        }
    }
}
