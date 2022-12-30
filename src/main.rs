use dotenvy;
use std::{env, fs};

use kindle_to_notion::{clippings, notion};

fn main() {
    // TODO Get config from file instead? inspo from: https://github.com/jakeswenson/notion/blob/main/examples/todo/main.rs
    dotenvy::dotenv().unwrap();

    // Reading the clippings
    let clippings_text = fs::read_to_string(
        // TODO FIX THIS -> USE THE PROPER VALUE
        // env::var("CLIPPINGS_FILE").expect("CLIPPINGS_FILE env variable not set"),
        "documents/My Clippings.txt",
    )
    .expect("Clippings file not found");

    let books_clips = clippings::parse_clips(clippings_text.as_str());

    // Reading the environment variables
    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY env variable not set");
    let parent_page_id = env::var("NOTION_PAGE_ID").expect("NOTION_PAGE_ID env variable not set");

    println!("STARTING UPLOAD of {} books", books_clips.len());

    // Uploading to Notion
    notion::upload_to_notion(api_key, parent_page_id, books_clips)
        .expect("Failed to upload to Notion");

    println!("ENDING UPLOAD");
}
