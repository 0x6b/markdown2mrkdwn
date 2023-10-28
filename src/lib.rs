use std::error::Error;
use std::ops::Add;

use markdown::{mdast::Node, to_mdast, ParseOptions};

struct Mrkdwn<'a> {
    text: &'a str,
    indent_level: usize,
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
                BlockQuote(blockquote) => {
                    format!("> {}", self.mrkdwnify(&blockquote.children))
                }
                Break(_) => "\n".to_string(),
                Code(code) => {
                    format!("```\n{}\n```", code.value)
                }
                Delete(delete) => {
                    format!("~{}~", self.mrkdwnify(&delete.children))
                }
                Emphasis(emphasis) => {
                    format!("_{}_", self.mrkdwnify(&emphasis.children))
                }
                Heading(heading) => {
                    format!("*{}*\n\n", self.mrkdwnify(&heading.children))
                }
                InlineCode(inline_code) => {
                    format!("`{}`", inline_code.value)
                }
                Link(link) => {
                    format!("<{}|{}>", &link.url, self.mrkdwnify(&link.children))
                }
                List(list) => {
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
                        list.children.iter().fold(String::new(), |acc, list_item| {
                            format!(
                                "{}{}•   {}\n",
                                acc,
                                "    ".repeat(self.indent_level - 1),
                                self.mrkdwnify(list_item.children().unwrap())
                            )
                        })
                    };
                    self.indent_level = self.indent_level.saturating_sub(1);
                    res.replace("\n\n", "\n").add("\n").to_string()
                }
                ListItem(list_item) => self.mrkdwnify(&list_item.children).to_string(),
                Paragraph(p) => {
                    format!("{}\n", self.mrkdwnify(&p.children))
                }
                Strong(strong) => {
                    format!("*{}*", self.mrkdwnify(&strong.children))
                }
                Text(text) => text.value.to_string(),
                ThematicBreak(_) => "\n----------\n".to_string(),
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
}
