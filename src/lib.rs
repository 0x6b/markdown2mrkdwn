use std::error::Error;
use std::ops::Add;

use markdown::mdast::List;
use markdown::mdast::Node::ListItem;
use markdown::{mdast::Node, to_mdast, ParseOptions};
use serde_json::json;

struct Mrkdwn<'a> {
    text: &'a str,
    indent_level: usize,
}

#[derive(Debug)]
enum Block {
    Header(String),
    Divider,
    Section(String),
}

impl Block {
    fn into_json(self) -> serde_json::Value {
        match self {
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
        }
    }
}

impl<'a> TryFrom<Mrkdwn<'a>> for Vec<Block> {
    type Error = Box<dyn Error>;

    fn try_from(mut v: Mrkdwn<'a>) -> Result<Self, Self::Error> {
        match to_mdast(v.text, &ParseOptions::gfm())?.children() {
            None => Err("no input?".into()),
            Some(ast) => Ok(v.mrkdwn_blockify(ast)),
        }
    }
}

impl<'a> TryFrom<Mrkdwn<'a>> for String {
    type Error = Box<dyn Error>;

    fn try_from(mut v: Mrkdwn<'a>) -> Result<Self, Self::Error> {
        match to_mdast(v.text, &ParseOptions::gfm())?.children() {
            None => Err("no input?".into()),
            Some(ast) => Ok(v
                .mrkdwnify(ast)
                .trim()
                .replace('"', "\\\"")
                .replace('&', "&amp;")
                .replace('\n', "\\n")
                .trim_end_matches("\\n")
                .to_string()),
        }
    }
}

impl<'a> Mrkdwn<'a> {
    pub fn from(text: &'a str) -> Self {
        Self { text, indent_level: 0 }
    }

    fn mrkdwnify(&mut self, node: &[Node]) -> String {
        use Node::*;

        node.iter()
            .map(|child| match child {
                BlockQuote(n) => self.surround_nodes_with(&n.children, "> ", ""),
                Break(_) => "\n".to_string(),
                Code(n) => Self::surround_with(&n.value, "```\n", "\n```"),
                Delete(n) => self.surround_nodes_with(&n.children, "~", "~"),
                Emphasis(n) => self.surround_nodes_with(&n.children, "_", "_"),
                Heading(n) => self.surround_nodes_with(&n.children, "*", "*\n\n"),
                InlineCode(n) => Self::surround_with(&n.value, "`", "`"),
                Link(n) => format!("<{}|{}>", &n.url, self.mrkdwnify(&n.children)),
                List(n) => self.handle_list(n),
                ListItem(n) => self.mrkdwnify(&n.children).to_string(),
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

    fn mrkdwn_blockify(&mut self, node: &[Node]) -> Vec<Block> {
        use Block::*;
        use Node::*;

        node.iter()
            .map(|child| match child {
                BlockQuote(n) => Section(self.surround_nodes_with(&n.children, "> ", "")),
                Break(_) => Section("\n".to_string()),
                Code(n) => Section(Self::surround_with(&n.value, "```\n", "\n```")),
                Delete(n) => Section(self.surround_nodes_with(&n.children, "~", "~")),
                Emphasis(n) => Section(self.surround_nodes_with(&n.children, "_", "_")),
                Heading(n) => Header(self.mrkdwnify(&n.children)),
                InlineCode(n) => Section(Self::surround_with(&n.value, "`", "`")),
                Link(n) => Section(format!("<{}|{}>", &n.url, self.mrkdwnify(&n.children))),
                List(n) => Section(self.handle_list(n)),
                ListItem(n) => Section(self.mrkdwnify(&n.children).to_string()),
                Paragraph(n) => Section(self.surround_nodes_with(&n.children, "", "\n")),
                Strong(n) => Section(self.surround_nodes_with(&n.children, "*", "*")),
                Text(n) => Section(n.value.to_string()),
                ThematicBreak(_) => Divider,
                _ => Section("".to_string()),
            })
            .collect::<Vec<_>>()
    }

    fn surround_with(s: &str, prefix: &str, suffix: &str) -> String {
        format!("{}{}{}", prefix, s, suffix)
    }

    fn surround_nodes_with(&mut self, node: &[Node], prefix: &str, suffix: &str) -> String {
        format!("{}{}{}", prefix, self.mrkdwnify(node), suffix)
    }

    fn handle_list(&mut self, list: &List) -> String {
        self.indent_level = self.indent_level.saturating_add(1);
        let res = if list.ordered {
            list.children
                .iter()
                .enumerate()
                .fold((0, String::new()), |(i, acc), (_, list_item)| {
                    (
                        i + 1,
                        acc + &format!(
                            "{}{}.  {}\n",
                            "    ".repeat(self.indent_level - 1),
                            i + 1,
                            self.mrkdwnify(list_item.children().unwrap())
                        ),
                    )
                })
                .1
        } else {
            list.children
                .iter()
                .enumerate()
                .fold((0, String::new()), |(i, acc), (_, list_item)| {
                    (i + 1, {
                        let task_list = match list.children.get(i) {
                            Some(ListItem(item)) => item.checked,
                            _ => None,
                        };
                        format!(
                            "{}{}{}   {}\n",
                            acc,
                            "    ".repeat(self.indent_level - 1),
                            match task_list {
                                None => "•",
                                Some(true) => "\u{2611}",  // ☑︎
                                Some(false) => "\u{2610}", // ☐
                            },
                            self.mrkdwnify(list_item.children().unwrap())
                        )
                    })
                })
                .1
        };
        self.indent_level = self.indent_level.saturating_sub(1);
        res.replace("\n\n", "\n").add("\n").to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::Mrkdwn;

    #[test]
    fn test_escaping() {
        [("&", "&amp;"), ("\"", "\\\"")].iter().for_each(|(input, expected)| {
            assert_eq!(String::try_from(Mrkdwn::from(input)).unwrap(), expected.to_string());
        });
    }

    #[test]
    fn test_visual_basics() {
        [
            ("*emphasis*", "_emphasis_"),
            ("_italic_", "_italic_"),
            ("**bold**", "*bold*"),
            ("~~strikethrough~~", "~strikethrough~"),
        ]
        .iter()
        .for_each(|(input, expected)| {
            assert_eq!(String::try_from(Mrkdwn::from(input)).unwrap(), expected.to_string());
        });
    }

    #[test]
    fn test_line_breaks() {
        assert_eq!(
            String::try_from(Mrkdwn::from("This is a line of text.\nAnd this is another one.")).unwrap(),
            "This is a line of text.\\nAnd this is another one.".to_string()
        );
    }

    #[test]
    fn test_blockquotes() {
        assert_eq!(
            String::try_from(Mrkdwn::from("This is unquoted.\n> This is quoted.")).unwrap(),
            "This is unquoted.\\n> This is quoted.".to_string()
        );
    }

    #[test]
    fn test_inline_code() {
        assert_eq!(
            String::try_from(Mrkdwn::from("This is `**inline code**`.")).unwrap(),
            "This is `**inline code**`.".to_string()
        );
    }

    #[test]
    fn test_code_blocks() {
        assert_eq!(
            String::try_from(Mrkdwn::from("```\nconsole.log('Hello, mrkdwn!')\n```")).unwrap(),
            "```\\nconsole.log('Hello, mrkdwn!')\\n```".to_string()
        );
    }

    #[test]
    fn test_links() {
        assert_eq!(
            String::try_from(Mrkdwn::from("[Slack](https://slack.com/)")).unwrap(),
            "<https://slack.com/|Slack>".to_string()
        );
    }

    #[test]
    fn test_lists() {
        assert_eq!(
            String::try_from(Mrkdwn::from("- First\n- Second\n- Third")).unwrap(),
            "•   First\\n•   Second\\n•   Third".to_string()
        );
    }

    #[test]
    fn test_thematic_breaks() {
        assert_eq!(String::try_from(Mrkdwn::from("---")).unwrap(), "----------".to_string());
    }

    #[test]
    fn test_task_lists() {
        assert_eq!(
            String::try_from(Mrkdwn::from("- [ ] First\n- [x] Second\n- [ ] Third")).unwrap(),
            "\u{2610}   First\\n\u{2611}   Second\\n\u{2610}   Third".to_string()
        );
    }

    #[test]
    fn test() {
        let md = r#"# Heading 1
## Heading 2
### Heading 3

Hello, ~~Markdown~~ **mrkdwn**! and _markdown_.

`mrkdwn` is text formatting markup style in [Slack](https://slack.com/).

---

- First
    - Second
        - Third
    - Fourth
        - Fifth
        - Sixth
- Seventh


1. Ordered list 1
    - Ordered list 1-1
        - Ordered list 1-2
1. Ordered list 2
    1. Ordered list 2-1
    1. Ordered list 2-2
1. Ordered list 3

> *This is blockquote.*

```
console.log('Hello, mrkdwn!')
```
"#;
        assert_eq!(
            String::try_from(Mrkdwn::from(md)).unwrap(),
            "*Heading 1*\\n\\n*Heading 2*\\n\\n*Heading 3*\\n\\nHello, ~Markdown~ *mrkdwn*! and _markdown_.\\n`mrkdwn` is text formatting markup style in <https://slack.com/|Slack>.\\n\\n----------\\n•   First\\n    •   Second\\n        •   Third\\n    •   Fourth\\n        •   Fifth\\n        •   Sixth\\n\\n•   Seventh\\n\\n1.  Ordered list 1\\n    •   Ordered list 1-1\\n        •   Ordered list 1-2\\n\\n2.  Ordered list 2\\n    1.  Ordered list 2-1\\n    2.  Ordered list 2-2\\n\\n3.  Ordered list 3\\n\\n> _This is blockquote._\\n```\\nconsole.log('Hello, mrkdwn!')\\n```",
                   );
    }

    #[test]
    fn test_blockify() {
        let md = r#"# Heading 1
## Heading 2
### Heading 3

Hello, ~~Markdown~~ **mrkdwn**! and _markdown_.

`mrkdwn` is text formatting markup style in [Slack](https://slack.com/).

---

- First
    - Second
        - Third
    - Fourth
        - Fifth
        - Sixth
- Seventh


1. Ordered list 1
    - Ordered list 1-1
        - Ordered list 1-2
1. Ordered list 2
    1. Ordered list 2-1
    1. Ordered list 2-2
1. Ordered list 3

> *This is blockquote.*

```
console.log('Hello, mrkdwn!')
```
"#;
        let blocks = Vec::try_from(Mrkdwn::from(md))
            .unwrap()
            .into_iter()
            .map(|block| block.into_json())
            .collect::<Vec<_>>();
        assert_eq!(
            format!("{{ blocks: {} }}", serde_json::to_string(&blocks).unwrap()),
            r#"{ blocks: [{"text":{"emoji":true,"text":"Heading 1","type":"plain_text"},"type":"header"},{"text":{"emoji":true,"text":"Heading 2","type":"plain_text"},"type":"header"},{"text":{"emoji":true,"text":"Heading 3","type":"plain_text"},"type":"header"},{"text":{"text":"Hello, ~Markdown~ *mrkdwn*! and _markdown_.\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"`mrkdwn` is text formatting markup style in <https://slack.com/|Slack>.\n","type":"mrkdwn"},"type":"section"},{"type":"divider"},{"text":{"text":"•   First\n    •   Second\n        •   Third\n    •   Fourth\n        •   Fifth\n        •   Sixth\n\n•   Seventh\n\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"1.  Ordered list 1\n    •   Ordered list 1-1\n        •   Ordered list 1-2\n\n2.  Ordered list 2\n    1.  Ordered list 2-1\n    2.  Ordered list 2-2\n\n3.  Ordered list 3\n\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"> _This is blockquote._\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"```\nconsole.log('Hello, mrkdwn!')\n```","type":"mrkdwn"},"type":"section"}] }"#
        );
    }
}
