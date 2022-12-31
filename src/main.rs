use std::path::PathBuf;
use std::{env, fs};

use kindle_to_notion::{clippings, notion};

fn main() {
    // For simplicity, the conf is read from a .env file at the moment
    dotenvy::dotenv().expect(".env file not found");

    // Reading the clippings
    // TODO Add a CLI option to specify a different clippings file
    let file_path: PathBuf = ["documents", "My Clippings.txt"].iter().collect();
    let clippings_text = fs::read_to_string(file_path).expect("{file_path} file not found");

    // Creating our clips data
    let books_clips = clippings::parse_clips(clippings_text.as_str());

    // Reading the environment variables for Notion
    let api_key = env::var("NOTION_API_KEY").expect("NOTION_API_KEY env variable not set");
    let parent_page_id = env::var("NOTION_PAGE_ID").expect("NOTION_PAGE_ID env variable not set");

    // Uploading to Notion
    notion::upload_to_notion(api_key, parent_page_id, books_clips)
        .expect("Failed to upload to Notion");

    // Archiving the clippings
    // TODO Add a CLI option to not archive the clippings
}
