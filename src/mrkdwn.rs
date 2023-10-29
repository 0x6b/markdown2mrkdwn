use std::{error::Error, ops::Add, sync::RwLock};

/// `Mrkdwn` is a public struct for handling GitHub Flavored Markdown text and its indentation level. Note that both
/// fields are not accessible from outside.
///
/// # Fields
///
/// - `text: &'a str` - A GitHub Flavored Markdown.
/// - `indent_level: usize` - Specifying the level of indentation.
pub struct Mrkdwn<'a> {
    /// Represents the markdown text.
    text: &'a str,

    /// Indication of the level of indentation.
    indent_level: RwLock<usize>,
}

impl<'a> Mrkdwn<'a> {
    /// Constructs a new instance of `Mrkdwn` with given text.
    ///
    /// # Arguments
    ///
    /// - `text` - A GitHub Flavored Markdown.
    pub fn from(text: &'a str) -> Self {
        Self {
            text,
            indent_level: RwLock::new(0),
        }
    }

    /// This method is responsible for markdownifying the text in `self`.
    ///
    /// # Returns
    ///
    /// - `Ok(String)`: If the process is successful, this method will return a markdownified version of `self.text`.
    /// - `Err(Box<dyn Error>)`: In case of an error during the process, it returns a boxed dynamic Error.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    ///
    /// - The text cannot be parsed into a Markdown abstract syntax tree.
    /// - The root node has no children elements.
    pub fn mrkdwnify(&self) -> Result<String, Box<dyn Error>> {
        Ok(self
            .transform_to_mrkdwn(
                markdown::to_mdast(self.text, &markdown::ParseOptions::gfm())
                    .map_err(|e| e.to_string())?
                    .children()
                    .ok_or("no input?")?,
            )
            .trim()
            .replace('"', "\\\"")
            .replace('&', "&amp;")
            .replace('\n', "\\n")
            .trim_end_matches("\\n")
            .to_string())
    }

    /// Converts the provided text into a Slack Block Kit blocks.
    /// It parses GitHub Flavored Markdown into AST and transforms each element.
    ///
    /// # Returns
    ///
    /// - `Ok(String)`: If the process is successful, this method will return a Slack blocks.
    /// - `Err(Box<dyn Error>)`: In case of an error during the process, it returns a boxed dynamic Error.
    ///
    /// # References
    ///
    /// - [Block Kit | Slack](https://api.slack.com/block-kit)
    pub fn blockify(&self) -> Result<String, Box<dyn Error>> {
        let blocks: Vec<serde_json::Value> = self
            .transform_to_blocks(
                markdown::to_mdast(self.text, &markdown::ParseOptions::gfm())
                    .map_err(|e| e.to_string())?
                    .children()
                    .ok_or("no input?")?,
            )?
            .into_iter()
            .flat_map(|block| block.try_into())
            .collect::<_>();

        Ok(format!(r#"{{ "blocks": {} }}"#, serde_json::to_string(&blocks)?))
    }

    fn transform_to_mrkdwn(&self, nodes: &[markdown::mdast::Node]) -> String {
        use markdown::mdast::Node::*;

        nodes
            .iter()
            .map(|child| match child {
                BlockQuote(n) => self.surround_nodes_with(&n.children, "> ", ""),
                Break(_) => "\n".to_string(),
                Code(n) => Self::surround_with(&n.value, "```\n", "\n```\n"),
                Delete(n) => self.surround_nodes_with(&n.children, "~", "~"),
                Emphasis(n) => self.surround_nodes_with(&n.children, "_", "_"),
                Heading(n) => self.surround_nodes_with(&n.children, "*", "*\n\n"),
                InlineCode(n) => Self::surround_with(&n.value, "`", "`"),
                Link(n) => format!("<{}|{}>", &n.url, self.transform_to_mrkdwn(&n.children)),
                List(n) => self.handle_list(n),
                ListItem(n) => self.transform_to_mrkdwn(&n.children).to_string(),
                Paragraph(n) => self.surround_nodes_with(&n.children, "", "\n"),
                Strong(n) => self.surround_nodes_with(&n.children, "*", "*"),
                Text(n) => n.value.to_string(),
                ThematicBreak(_) => "\n----------\n".to_string(),
                // not supported
                Definition(_)
                | FootnoteDefinition(_)
                | FootnoteReference(_)
                | Html(_)
                | Image(_)
                | ImageReference(_)
                | InlineMath(_)
                | LinkReference(_)
                | Math(_)
                | MdxFlowExpression(_)
                | MdxJsxFlowElement(_)
                | MdxJsxTextElement(_)
                | MdxTextExpression(_)
                | MdxjsEsm(_)
                | Root(_)
                | Table(_)
                | TableCell(_)
                | TableRow(_)
                | Toml(_)
                | Yaml(_) => "".to_string(),
            })
            .collect::<String>()
    }

    pub fn transform_to_blocks(
        &self,
        nodes: &[markdown::mdast::Node],
    ) -> Result<Vec<crate::block::Block>, Box<dyn Error>> {
        use crate::block::Block::*;
        use markdown::mdast::Node::*;

        Ok(nodes
            .iter()
            .flat_map(|child| match child {
                BlockQuote(n) => vec![Section(self.surround_nodes_with(&n.children, "> ", ""))],
                Break(_) => vec![Section("\n".to_string())],
                Code(n) => vec![Section(Self::surround_with(&n.value, "```\n", "\n```\n"))],
                Delete(n) => vec![Section(self.surround_nodes_with(&n.children, "~", "~"))],
                Emphasis(n) => vec![Section(self.surround_nodes_with(&n.children, "_", "_"))],
                Heading(n) => match n.depth {
                    1 => vec![Header(self.transform_to_mrkdwn(&n.children)), Divider],
                    _ => vec![Header(self.transform_to_mrkdwn(&n.children))],
                },
                InlineCode(n) => vec![Section(Self::surround_with(&n.value, "`", "`"))],
                Link(n) => vec![Section(format!(
                    "<{}|{}>",
                    &n.url,
                    self.transform_to_mrkdwn(&n.children)
                ))],
                List(n) => vec![Section(self.handle_list(n))],
                ListItem(n) => vec![Section(self.transform_to_mrkdwn(&n.children).to_string())],
                Paragraph(n) => vec![Section(self.surround_nodes_with(&n.children, "", "\n"))],
                Strong(n) => vec![Section(self.surround_nodes_with(&n.children, "*", "*"))],
                Text(n) => vec![Section(n.value.to_string())],
                ThematicBreak(_) => vec![Divider],
                _ => vec![Section("".to_string())],
            })
            .collect::<_>())
    }

    fn surround_with(s: &str, prefix: &str, suffix: &str) -> String {
        format!("{}{}{}", prefix, s, suffix)
    }

    fn surround_nodes_with(&self, nodes: &[markdown::mdast::Node], prefix: &str, suffix: &str) -> String {
        format!("{}{}{}", prefix, self.transform_to_mrkdwn(nodes), suffix)
    }

    fn handle_list(&self, list: &markdown::mdast::List) -> String {
        {
            let mut indent_level = self.indent_level.write().unwrap();
            *indent_level = indent_level.saturating_add(1);
        }

        let res = {
            list.children
                .iter()
                .enumerate()
                .fold((0, String::new()), |(i, acc), (_, list_item)| {
                    let indent = "    ".repeat({ *self.indent_level.read().unwrap() } - 1);
                    let prefix = if list.ordered {
                        format!("{}.  ", i + 1)
                    } else {
                        let task_list = match list.children.get(i) {
                            Some(markdown::mdast::Node::ListItem(item)) => item.checked,
                            _ => None,
                        };

                        format!(
                            "{}   ",
                            match task_list {
                                None => "•",
                                Some(true) => "\u{2611}",  // ☑︎
                                Some(false) => "\u{2610}", // ☐
                            }
                        )
                    };

                    (
                        i + 1,
                        format!(
                            "{}{}{}{}\n",
                            acc,
                            indent,
                            prefix,
                            self.transform_to_mrkdwn(list_item.children().unwrap())
                        ),
                    )
                })
                .1
        }
        .replace("\n\n", "\n")
        .add("\n");

        {
            let mut indent_level = self.indent_level.write().unwrap();
            *indent_level = indent_level.saturating_sub(1);
        }

        res
    }
}
