use std::fmt::{Display, Formatter, Result};

use serde_json::{Value, json};

use crate::Block::{Divider, Header, Section, Table};

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Block {
    Header(String),
    Divider,
    Section(String),
    /// A [table block](https://docs.slack.dev/reference/block-kit/blocks/table-block/).
    ///
    /// - `column_settings`: per-column alignment (`left`/`center`/`right`). `None` keeps the
    ///   default (left) alignment for that column.
    /// - `rows`: each row is a list of pre-built cell values (`raw_text` or `rich_text`).
    Table {
        column_settings: Vec<Option<String>>,
        rows: Vec<Vec<Value>>,
    },
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
            Table { column_settings, rows } => {
                let mut table = json!({
                    "type": "table",
                    "rows": rows,
                });

                // Trim trailing columns that use the default alignment: Slack applies
                // `column_settings` to the leading columns only, so they can be omitted.
                let last = column_settings.iter().rposition(Option::is_some);
                if let Some(last) = last {
                    let settings: Vec<Value> = column_settings[..=last]
                        .iter()
                        .map(|align| match align {
                            Some(align) => json!({ "align": align }),
                            None => Value::Null,
                        })
                        .collect();
                    table["column_settings"] = Value::Array(settings);
                }

                table
            }
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Header(text) => write!(f, "Header: {text}"),
            Divider => write!(f, "----------"),
            Section(text) => write!(f, "Section: {text}"),
            Table { rows, .. } => write!(f, "Table: {} rows", rows.len()),
        }
    }
}
