# markdown2mrkdwn

A library which converts GitHub Flavored Markdown to Slack's mrkdwn or blocks. Not perfect, but it'll be enough for my use cases.

## Usage

1. Run the example.
    ```console
    $ cargo run --examples blocks
    ```
2. Copy the output and paste it to [Block Kit Builder](https://app.slack.com/block-kit-builder/).
3. See the preview to check if it works as expected.

## Reference

- [Formatting text for app surfaces | Slack](https://api.slack.com/reference/surfaces/formatting)

## License

MIT. See [LICENSE](LICENSE) for details.

## Privacy

The conversion is solely done locally. The crate never sends user action/data to any server.
