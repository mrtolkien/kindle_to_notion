use std::str::FromStr;

use crate::clippings::Clip;
use anyhow::Result;
use reqwest;
use reqwest::header::{HeaderMap, HeaderName};
use serde_json::json;

const NOTION_API_URL: &str = "https://api.notion.com/v1/pages";

const NOTION_API_VERSION_HEADER: &str = "Notion-Version";
const NOTION_API_VERSION: &str = "2022-06-28";

// Creating our clip function that uses the parent
impl Clip {
    fn to_notion_body(&self, parent_page_id: String) -> serde_json::Value {
        json!({
            "parent": {
                "page_id": parent_page_id
            },
            "properties": {
                "title": [
                        {
                            "text": {
                                "content": self.book
                            }
                        }
                    ]
            },
            // TODO Re-add
            "children": [
                // Adding author as a header
                {
                    "object": "block",
                    "type": "heading_2",
                    "heading_2": {
                        "rich_text": [
                            {
                                "type": "text",
                                "text": {
                                    "content": self.author
                                }
                            }
                        ]
                    }
                },
                {
                    "object": "block",
                    "type": "paragraph",
                    "paragraph": {
                        "rich_text": [
                            {
                                "type": "text",
                                "text": {
                                    "content": format!("{}\n\t{}", self.content, self.date)
                                }
                            }
                        ]
                    }
                }]
        })
    }
}

pub fn upload_to_notion(api_key: String, parent_page_id: String, clips: Vec<Clip>) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    // TODO Cleanup if that works
    let mut headers = HeaderMap::new();
    let header_name = HeaderName::from_str(NOTION_API_VERSION_HEADER)?;
    headers.insert(header_name, NOTION_API_VERSION.parse()?);

    for clip in clips {
        // TODO Group clips from the same book together
        let res = client
            .post(NOTION_API_URL)
            .bearer_auth(api_key.clone())
            .headers(headers.clone())
            .json(&clip.to_notion_body(parent_page_id.clone()))
            .send()?;

        println!("{res:?}");
    }

    Ok(())
}
