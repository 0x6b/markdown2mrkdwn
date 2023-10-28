use markdown2mrkdwn::Mrkdwn;

fn main() {
    println!(
        "{}",
        Mrkdwn::from("`mrkdwn` is text formatting markup style in [Slack](https://slack.com/).")
            .mrkdwnify()
            .unwrap()
    );
}
