use markdown2mrkdwn::Mrkdwn;

fn main() {
    println!(
        "{}",
        Mrkdwn::from(std::fs::read_to_string("examples/sample.md").unwrap().as_str())
            .mrkdwnify()
            .unwrap()
    );
}
