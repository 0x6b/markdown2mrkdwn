use anyhow::{Result, anyhow};
use markdown::{
    ParseOptions,
    mdast::{List, Node},
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

impl<'a> Mrkdwn<'a> {
    /// Constructs a new instance of `Mrkdwn` with given text.
    ///
    /// # Arguments
    ///
    /// - `text` - A GitHub Flavored Markdown.
    pub fn from(text: &'a str) -> Self {
        Self { text }
    }

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
                Node::Paragraph(n) => {
                    vec![Section(self.surround_nodes_with(&n.children, "", "\n", 0))]
                }
                Node::Strong(n) => {
                    vec![Section(self.surround_nodes_with(&n.children, "*", "*", 0))]
                }
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
}
