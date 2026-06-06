use std::fs::read_to_string;

use markdown2mrkdwn::Mrkdwn;

fn main() {
    println!(
        "{}",
        Mrkdwn::from(read_to_string("examples/sample.md").unwrap().as_str())
            .blocks_stringify()
            .unwrap()
    );
}
