use anyhow::{Result, anyhow};
use markdown::{
    ParseOptions,
    mdast::{AlignKind, Image, List, Node, Table},
    to_mdast,
};
use serde_json::{Value, to_string};

use crate::Block;

/// `Mrkdwn` is a public struct for handling GitHub Flavored Markdown text.
/// Note that the `text` field is not accessible from outside.
///
/// # Fields
///
/// - `text: &'a str` - A GitHub Flavored Markdown.
pub struct Mrkdwn<'a> {
    /// Represents the markdown text.
    text: &'a str,
}

impl<'a> From<&'a str> for Mrkdwn<'a> {
    /// Constructs a new instance of `Mrkdwn` from the given GitHub Flavored Markdown text.
    fn from(text: &'a str) -> Self {
        Self { text }
    }
}

impl<'a> Mrkdwn<'a> {
    /// This method is responsible for markdownifying the text in `self`.
    ///
    /// # Returns
    ///
    /// - `Ok(String)`: If the process is successful, this method will return a markdownified
    ///   version of `self.text`.
    /// - `Err(String)`: In case of an error during the process, it returns an Error.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// - The text cannot be parsed into a Markdown abstract syntax tree.
    /// - The root node has no children elements.
    pub fn mrkdwnify(&self) -> Result<String> {
        let ast = to_mdast(self.text, &ParseOptions::gfm())
            .map_err(|e| anyhow!("Failed to parse markdown: {e}"))?;

        let root = ast.children().ok_or_else(|| anyhow!("no input?"))?;

        let mut result = self.transform_to_mrkdwn(root);
        result = result
            .trim()
            .replace('"', "\\\"")
            .replace('&', "&amp;")
            .replace('\n', "\\n")
            .trim_end_matches("\\n")
            .to_string();

        Ok(result)
    }

    /// Converts the provided text into a Slack Block Kit blocks.
    /// It parses GitHub Flavored Markdown into AST and transforms each element.
    ///
    /// # Returns
    ///
    /// - `Ok(String)`: If the process is successful, this method will return a Slack blocks.
    /// - `Err(String)`: In case of an error during the process, it returns an Error.
    ///
    /// # References
    ///
    /// - [Block Kit | Slack](https://api.slack.com/block-kit)
    pub fn blocks_stringify(&self) -> Result<String> {
        let blocks: Vec<Value> = self.blockify()?.into_iter().map(Value::from).collect();
        Ok(format!(r#"{{ "blocks": {} }}"#, to_string(&blocks)?))
    }

    /// Converts the provided text into a Slack Block Kit blocks.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<Block>)`: If the process is successful, this method will return a Vec of Block.
    /// - `Err(String)`: In case of an error during the process, it returns an Error.
    pub fn blockify(&self) -> Result<Vec<Block>> {
        let ast = to_mdast(self.text, &ParseOptions::gfm())
            .map_err(|e| anyhow!("Failed to parse markdown: {e}"))?;

        let root = ast.children().ok_or_else(|| anyhow!("no input?"))?;

        self.transform_to_blocks(root)
    }

    fn transform_to_mrkdwn(&self, nodes: &[Node]) -> String {
        self.transform_to_mrkdwn_with_indent(nodes, 0)
    }

    fn transform_to_mrkdwn_with_indent(&self, nodes: &[Node], indent_level: usize) -> String {
        nodes
            .iter()
            .map(|child| match child {
                Node::Blockquote(n) => {
                    self.surround_nodes_with(&n.children, "> ", "", indent_level)
                }
                Node::Break(_) => "\n".to_string(),
                Node::Code(n) => Self::surround_with(&n.value, "```\n", "\n```\n"),
                Node::Delete(n) => self.surround_nodes_with(&n.children, "~", "~", indent_level),
                Node::Emphasis(n) => self.surround_nodes_with(&n.children, "_", "_", indent_level),
                Node::Heading(n) => {
                    self.surround_nodes_with(&n.children, "*", "*\n\n", indent_level)
                }
                Node::InlineCode(n) => Self::surround_with(&n.value, "`", "`"),
                Node::Link(n) => format!(
                    "<{}|{}>",
                    &n.url,
                    self.transform_to_mrkdwn_with_indent(&n.children, indent_level)
                ),
                Node::List(n) => self.handle_list(n, indent_level),
                Node::ListItem(n) => {
                    self.transform_to_mrkdwn_with_indent(&n.children, indent_level)
                }
                Node::Paragraph(n) => self.surround_nodes_with(&n.children, "", "\n", indent_level),
                Node::Strong(n) => self.surround_nodes_with(&n.children, "*", "*", indent_level),
                Node::Text(n) => n.value.clone(),
                Node::ThematicBreak(_) => "\n----------\n".to_string(),
                _ => String::new(),
            })
            .collect()
    }

    fn transform_to_blocks(&self, nodes: &[Node]) -> Result<Vec<Block>> {
        use crate::block::Block::*;

        Ok(nodes
            .iter()
            .flat_map(|child| match child {
                Node::Blockquote(n) => {
                    vec![Section(self.surround_nodes_with(&n.children, "> ", "", 0))]
                }
                Node::Break(_) => vec![Section("\n".to_string())],
                Node::Code(n) => vec![Section(Self::surround_with(&n.value, "```\n", "\n```\n"))],
                Node::Delete(n) => {
                    vec![Section(self.surround_nodes_with(&n.children, "~", "~", 0))]
                }
                Node::Emphasis(n) => {
                    vec![Section(self.surround_nodes_with(&n.children, "_", "_", 0))]
                }
                Node::Heading(n) => {
                    let text = self.transform_to_mrkdwn(&n.children);
                    match n.depth {
                        1 => vec![Header(text), Divider],
                        2 => vec![Header(text)],
                        _ => vec![Section(self.surround_nodes_with(&n.children, "*", "*", 0))],
                    }
                }
                Node::InlineCode(n) => vec![Section(Self::surround_with(&n.value, "`", "`"))],
                Node::Link(n) => {
                    vec![Section(format!("<{}|{}>", &n.url, self.transform_to_mrkdwn(&n.children)))]
                }
                Node::List(n) => vec![Section(self.handle_list(n, 0))],
                Node::ListItem(n) => vec![Section(self.transform_to_mrkdwn(&n.children))],
                Node::Paragraph(n) => self.handle_paragraph(&n.children),
                Node::Strong(n) => {
                    vec![Section(self.surround_nodes_with(&n.children, "*", "*", 0))]
                }
                Node::Table(n) => vec![Self::handle_table(n)],
                Node::Text(n) => vec![Section(n.value.clone())],
                Node::ThematicBreak(_) => vec![Divider],
                _ => vec![],
            })
            .collect())
    }

    fn surround_with(s: &str, prefix: &str, suffix: &str) -> String {
        format!("{prefix}{s}{suffix}")
    }

    fn surround_nodes_with(
        &self,
        nodes: &[Node],
        prefix: &str,
        suffix: &str,
        indent_level: usize,
    ) -> String {
        format!("{prefix}{}{suffix}", self.transform_to_mrkdwn_with_indent(nodes, indent_level))
    }

    fn handle_list(&self, list: &List, indent_level: usize) -> String {
        let indent = "    ".repeat(indent_level);
        list.children
            .iter()
            .enumerate()
            .fold(String::new(), |acc, (i, item)| {
                let prefix = if list.ordered {
                    format!("{}.  ", i + 1)
                } else {
                    let checked = list
                        .children
                        .get(i)
                        .and_then(|n| if let Node::ListItem(li) = n { li.checked } else { None });
                    format!(
                        "{}   ",
                        match checked {
                            None => "•",
                            Some(true) => "\u{2611}",
                            Some(false) => "\u{2610}",
                        }
                    )
                };

                let children = item.children().map_or(&[][..], |v| v.as_slice());
                format!(
                    "{acc}{indent}{prefix}{}\n",
                    self.transform_to_mrkdwn_with_indent(children, indent_level + 1)
                )
            })
            .replace("\n\n", "\n")
            + "\n"
    }

    /// Converts a Markdown table into a Slack [table block].
    ///
    /// Each cell is rendered as a `rich_text` cell so that inline formatting (bold, italic,
    /// strikethrough, inline code, and links) is preserved.
    ///
    /// [table block]: https://docs.slack.dev/reference/block-kit/blocks/table-block/
    fn handle_table(table: &Table) -> Block {
        let column_settings = table
            .align
            .iter()
            .map(|align| match align {
                AlignKind::Left => Some("left".to_string()),
                AlignKind::Center => Some("center".to_string()),
                AlignKind::Right => Some("right".to_string()),
                AlignKind::None => None,
            })
            .collect();

        let rows = table
            .children
            .iter()
            .filter_map(|row| match row {
                Node::TableRow(row) => Some(
                    row.children
                        .iter()
                        .map(|cell| match cell {
                            Node::TableCell(cell) => Self::table_cell(&cell.children),
                            _ => Self::table_cell(&[]),
                        })
                        .collect(),
                ),
                _ => None,
            })
            .collect();

        Block::Table { column_settings, rows }
    }

    /// Converts a Markdown paragraph into one or more blocks.
    ///
    /// Images are lifted into their own [image block]s, while the surrounding inline content is
    /// kept as `section` blocks. A paragraph with no images becomes a single `section`.
    ///
    /// [image block]: https://docs.slack.dev/reference/block-kit/blocks/image-block/
    fn handle_paragraph(&self, nodes: &[Node]) -> Vec<Block> {
        if !nodes.iter().any(|node| matches!(node, Node::Image(_))) {
            return vec![Block::Section(self.surround_nodes_with(nodes, "", "\n", 0))];
        }

        let mut blocks = Vec::new();
        let mut buffer: Vec<Node> = Vec::new();
        let flush = |buffer: &mut Vec<Node>, blocks: &mut Vec<Block>| {
            if !buffer.is_empty() {
                let text = self.transform_to_mrkdwn(buffer);
                if !text.trim().is_empty() {
                    blocks.push(Block::Section(format!("{text}\n")));
                }
                buffer.clear();
            }
        };

        for node in nodes {
            match node {
                Node::Image(image) => {
                    flush(&mut buffer, &mut blocks);
                    blocks.push(Self::image_block(image));
                }
                other => buffer.push(other.clone()),
            }
        }
        flush(&mut buffer, &mut blocks);

        blocks
    }

    /// Builds an image block from a Markdown image node.
    ///
    /// Slack requires a non-empty `alt_text`, so the Markdown alt text is used when present,
    /// falling back to the image title and finally the URL.
    fn image_block(image: &Image) -> Block {
        let title = image.title.clone().filter(|title| !title.trim().is_empty());
        let alt_text = if image.alt.trim().is_empty() {
            title.clone().unwrap_or_else(|| image.url.clone())
        } else {
            image.alt.clone()
        };
        Block::Image { url: image.url.clone(), alt_text, title }
    }

    /// Builds a single `rich_text` table cell from inline Markdown nodes.
    fn table_cell(nodes: &[Node]) -> Value {
        let mut elements = Self::rich_text_elements(nodes, Style::default());
        // Slack rejects a `rich_text_section` with no elements, so emit an empty text element.
        if elements.is_empty() {
            elements.push(Self::text_element("", Style::default()));
        }
        serde_json::json!({
        "type": "rich_text",
        "elements": [ { "type": "rich_text_section", "elements": elements } ],
        })
    }

    /// Recursively converts inline Markdown nodes into Slack `rich_text` section elements,
    /// carrying the active text style through nested formatting nodes.
    fn rich_text_elements(nodes: &[Node], style: Style) -> Vec<Value> {
        let mut elements = Vec::new();
        for node in nodes {
            match node {
                Node::Text(n) => elements.push(Self::text_element(&n.value, style)),
                Node::Strong(n) => {
                    elements.extend(Self::rich_text_elements(&n.children, style.bold()))
                }
                Node::Emphasis(n) => {
                    elements.extend(Self::rich_text_elements(&n.children, style.italic()))
                }
                Node::Delete(n) => {
                    elements.extend(Self::rich_text_elements(&n.children, style.strike()))
                }
                Node::InlineCode(n) => elements.push(Self::text_element(&n.value, style.code())),
                Node::Break(_) => elements.push(Self::text_element("\n", style)),
                Node::Link(n) => {
                    let mut element = serde_json::json!({
                        "type": "link",
                        "url": n.url,
                        "text": Self::plain_text(&n.children),
                    });
                    if let Some(value) = style.to_value() {
                        element["style"] = value;
                    }
                    elements.push(element);
                }
                _ => {}
            }
        }
        elements
    }

    /// Builds a `text` element, attaching a `style` object only when some style is active.
    fn text_element(text: &str, style: Style) -> Value {
        let mut element = serde_json::json!({ "type": "text", "text": text });
        if let Some(value) = style.to_value() {
            element["style"] = value;
        }
        element
    }

    /// Flattens inline nodes into plain text, used for the `text` of a `rich_text` link element
    /// (Slack link elements take a plain string, not nested formatting).
    fn plain_text(nodes: &[Node]) -> String {
        nodes
            .iter()
            .map(|node| match node {
                Node::Text(n) => n.value.clone(),
                Node::InlineCode(n) => n.value.clone(),
                Node::Strong(n) => Self::plain_text(&n.children),
                Node::Emphasis(n) => Self::plain_text(&n.children),
                Node::Delete(n) => Self::plain_text(&n.children),
                _ => String::new(),
            })
            .collect()
    }
}

/// Active inline text style while building Slack `rich_text` elements.
#[derive(Clone, Copy, Default)]
struct Style {
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
}

impl Style {
    fn bold(self) -> Self {
        Self { bold: true, ..self }
    }

    fn italic(self) -> Self {
        Self { italic: true, ..self }
    }

    fn strike(self) -> Self {
        Self { strike: true, ..self }
    }

    fn code(self) -> Self {
        Self { code: true, ..self }
    }

    /// Serializes to a Slack style object, or `None` when no style is active so the `style`
    /// key can be omitted entirely.
    fn to_value(self) -> Option<Value> {
        if !(self.bold || self.italic || self.strike || self.code) {
            return None;
        }
        let mut value = serde_json::Map::new();
        if self.bold {
            value.insert("bold".to_string(), Value::Bool(true));
        }
        if self.italic {
            value.insert("italic".to_string(), Value::Bool(true));
        }
        if self.strike {
            value.insert("strike".to_string(), Value::Bool(true));
        }
        if self.code {
            value.insert("code".to_string(), Value::Bool(true));
        }
        Some(Value::Object(value))
    }
}
