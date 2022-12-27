use dotenv::dotenv;
use std::{env, fs};

use kindle_to_notion::{clippings, notion};

fn main() {
    // TODO GET config + env variables, inspo from: https://github.com/jakeswenson/notion/blob/main/examples/todo/main.rs
    dotenv().ok();

    // Reading the clippings
    // TODO Get file location from config
    let clippings_text =
        fs::read_to_string("tests/data/clippings.txt").expect("Test file not found");
    let clips = clippings::parse_clips(clippings_text.as_str());

    // Reading the environment variables
    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY not set");
    let parent_page_id = env::var("NOTION_PAGE_ID").expect("NOTION_PAGE_ID not set");

    // Uploading to Notion
    notion::upload_to_notion(api_key, parent_page_id, clips).expect("Failed to upload to Notion");
}
