use markdown2mrkdwn::Mrkdwn;

fn main() {
    println!(
        "{}",
        Mrkdwn::from(
            "# heading1 \n## heading 2\n- First\n    - Second\n- Third\n```\n$ cargo run --example main\n```\n"
        )
        .blockify()
        .unwrap()
    );
}
