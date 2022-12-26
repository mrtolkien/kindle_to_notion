use std::fs;

use kindle_to_notion::{clippings, notion};

fn main() {
    let clippings = fs::read_to_string("tests/data/clippings.txt").expect("Test file not found");
    let clips = clippings::parse_clips(clippings.as_str());

    notion::upload_to_notion(clips);
}
