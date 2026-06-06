use std::fs::read_to_string;
fn main() {
    println!(
        "{}",
        markdown2mrkdwn::Mrkdwn::from(read_to_string("examples/sample.md").unwrap().as_str())
            .mrkdwnify()
            .unwrap()
    );
}
