pub use block::Block;
pub use mrkdwn::Mrkdwn;

mod block;
mod mrkdwn;

#[cfg(test)]
mod test {
    mod blocks_stringify {
        use crate::Mrkdwn;

        macro_rules! test {
            ($name:ident, $input:expr, $expected:expr) => {
                #[test]
                fn $name() {
                    let mrkdwn = &Mrkdwn::from($input).blocks_stringify().unwrap();
                    assert_eq!(
                        serde_json::from_str::<serde_json::Value>(&mrkdwn).unwrap(),
                        serde_json::from_str::<serde_json::Value>($expected).unwrap()
                    );
                }
            };
        }

        test!(
            escaping1,
            "&",
            r#"{ "blocks": [ { "text": { "text": "&\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            escaping2,
            "\"",
            r#"{ "blocks": [ { "text": { "text": "\"\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            emphasis,
            "*emphasis*",
            r#"{ "blocks": [ { "text": { "text": "_emphasis_\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            italic,
            "_italic_",
            r#"{ "blocks": [ { "text": { "text": "_italic_\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            bold,
            "**bold**",
            r#"{ "blocks": [ { "text": { "text": "*bold*\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            strikethrough,
            "~~strikethrough~~",
            r#"{ "blocks": [ { "text": { "text": "~strikethrough~\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            line_breaks,
            "This is a line of text.\nAnd this is another one.",
            r#"{ "blocks": [ { "text": { "text": "This is a line of text.\nAnd this is another one.\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            blockquotes,
            "This is unquoted.\n> This is quoted.",
            r#"{ "blocks": [ { "text": { "text": "This is unquoted.\n", "type": "mrkdwn" }, "type": "section" }, { "text": { "text": "> This is quoted.\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            inline_code,
            "This is `**inline code**`.",
            r#"{ "blocks": [ { "text": { "text": "This is `**inline code**`.\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            code_blocks,
            "```\nconsole.log('Hello, mrkdwn!')\n```",
            r#"{ "blocks": [ { "text": { "text": "```\nconsole.log('Hello, mrkdwn!')\n```\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            headings,
            "# Heading 1\n## Heading 2\n### Heading 3",
            r#"{ "blocks": [ { "text": { "emoji": true, "text": "Heading 1", "type": "plain_text" }, "type": "header" }, { "type": "divider" }, { "text": { "emoji": true, "text": "Heading 2", "type": "plain_text" }, "type": "header" }, { "text": { "emoji": true, "text": "Heading 3", "type": "plain_text" }, "type": "header" } ] }"#
        );
        test!(
            link,
            "[Slack](https://slack.com/)",
            r#"{ "blocks": [ { "text": { "text": "<https://slack.com/|Slack>\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            lists,
            "- First\n- Second\n- Third",
            r#"{ "blocks": [ { "text": { "text": "•   First\n•   Second\n•   Third\n\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            ordered_lists,
            "1. First\n1. Second\n1. Third",
            r#"{ "blocks": [ { "text": { "text": "1.  First\n2.  Second\n3.  Third\n\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            nested_lists,
            "- First\n    - Second\n        - Third\n    - Fourth\n        - Fifth\n        - Sixth\n- Seventh",
            r#"{ "blocks": [ { "text": { "text": "•   First\n    •   Second\n        •   Third\n    •   Fourth\n        •   Fifth\n        •   Sixth\n\n•   Seventh\n\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(thematic_breaks, "---", r#"{ "blocks": [ { "type": "divider" } ] }"#);
        test!(
            task_lists,
            "- [ ] First\n- [x] Second\n- [ ] Third",
            r#"{ "blocks": [ { "text": { "text": "☐   First\n☑   Second\n☐   Third\n\n", "type": "mrkdwn" }, "type": "section" } ] }"#
        );
        test!(
            long_text,
            r#"# Heading 1
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

Another paragraph.
"#,
            r#"{ "blocks": [{"text":{"emoji":true,"text":"Heading 1","type":"plain_text"},"type":"header"},{"type":"divider"},{"text":{"emoji":true,"text":"Heading 2","type":"plain_text"},"type":"header"},{"text":{"emoji":true,"text":"Heading 3","type":"plain_text"},"type":"header"},{"text":{"text":"Hello, ~Markdown~ *mrkdwn*! and _markdown_.\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"`mrkdwn` is text formatting markup style in <https://slack.com/|Slack>.\n","type":"mrkdwn"},"type":"section"},{"type":"divider"},{"text":{"text":"•   First\n    •   Second\n        •   Third\n    •   Fourth\n        •   Fifth\n        •   Sixth\n\n•   Seventh\n\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"1.  Ordered list 1\n    •   Ordered list 1-1\n        •   Ordered list 1-2\n\n2.  Ordered list 2\n    1.  Ordered list 2-1\n    2.  Ordered list 2-2\n\n3.  Ordered list 3\n\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"> _This is blockquote._\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"```\nconsole.log('Hello, mrkdwn!')\n```\n","type":"mrkdwn"},"type":"section"},{"text":{"text":"Another paragraph.\n","type":"mrkdwn"},"type":"section"}] }"#
        );
    }

    mod mrkdwnify {
        use crate::Mrkdwn;

        macro_rules! test {
            ($name:ident, $input:expr, $expected:expr) => {
                #[test]
                fn $name() {
                    assert_eq!(
                        Mrkdwn::from($input).mrkdwnify().unwrap(),
                        $expected.to_string()
                    );
                }
            };
        }

        test!(escaping1, "&", "&amp;");
        test!(escaping2, "\"", "\\\"");
        test!(emphasis, "*emphasis*", "_emphasis_");
        test!(italic, "_italic_", "_italic_");
        test!(bold, "**bold**", "*bold*");
        test!(strikethrough, "~~strikethrough~~", "~strikethrough~");
        test!(
            line_breaks,
            "This is a line of text.\nAnd this is another one.",
            "This is a line of text.\\nAnd this is another one."
        );
        test!(
            blockquotes,
            "This is unquoted.\n> This is quoted.",
            "This is unquoted.\\n> This is quoted."
        );
        test!(inline_code, "This is `**inline code**`.", "This is `**inline code**`.");
        test!(
            code_blocks,
            "```\nconsole.log('Hello, mrkdwn!')\n```",
            "```\\nconsole.log('Hello, mrkdwn!')\\n```"
        );
        test!(
            headings,
            "# Heading 1\n## Heading 2\n### Heading 3",
            "*Heading 1*\\n\\n*Heading 2*\\n\\n*Heading 3*"
        );
        test!(link, "[Slack](https://slack.com/)", "<https://slack.com/|Slack>");
        test!(
            lists,
            "- First\n- Second\n- Third",
            "•   First\\n•   Second\\n•   Third"
        );
        test!(
            ordered_lists,
            "1. First\n1. Second\n1. Third",
            "1.  First\\n2.  Second\\n3.  Third"
        );
        test!(
            nested_lists,
            "- First\n    - Second\n        - Third\n    - Fourth\n        - Fifth\n        - Sixth\n- Seventh",
            "•   First\\n    •   Second\\n        •   Third\\n    •   Fourth\\n        •   Fifth\\n        •   Sixth\\n\\n•   Seventh"
        );
        test!(thematic_breaks, "---", "----------");
        test!(
            task_lists,
            "- [ ] First\n- [x] Second\n- [ ] Third",
            "\u{2610}   First\\n\u{2611}   Second\\n\u{2610}   Third"
        );
        test!(long_text, r#"# Heading 1
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

Another paragraph.
"#, "*Heading 1*\\n\\n*Heading 2*\\n\\n*Heading 3*\\n\\nHello, ~Markdown~ *mrkdwn*! and _markdown_.\\n`mrkdwn` is text formatting markup style in <https://slack.com/|Slack>.\\n\\n----------\\n•   First\\n    •   Second\\n        •   Third\\n    •   Fourth\\n        •   Fifth\\n        •   Sixth\\n\\n•   Seventh\\n\\n1.  Ordered list 1\\n    •   Ordered list 1-1\\n        •   Ordered list 1-2\\n\\n2.  Ordered list 2\\n    1.  Ordered list 2-1\\n    2.  Ordered list 2-2\\n\\n3.  Ordered list 3\\n\\n> _This is blockquote._\\n```\\nconsole.log('Hello, mrkdwn!')\\n```\\nAnother paragraph.");
    }
}
