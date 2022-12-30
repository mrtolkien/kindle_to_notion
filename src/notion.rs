use std::str::FromStr;

use crate::clippings::BookClips;
use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::{self, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const NOTION_API_URL: &str = "https://api.notion.com/v1/pages";

const NOTION_API_VERSION_HEADER: &str = "Notion-Version";
const NOTION_API_VERSION: &str = "2022-06-28";

#[derive(Debug, Serialize, Deserialize)]
struct NotionBookPage {
    parent: Value,
    icon: Value,
    properties: Value,
    children: Vec<Value>,
}

// Creating our clip function that uses the parent
impl BookClips {
    fn to_notion_body(&self, parent_page_id: String) -> NotionBookPage {
        // TODO Use strongly typed JSON everywhere to make it easier, it's a bit disgusting here
        let mut children = Vec::new();

        // We split on : if it's in the name, as it's usually ridiculously long books names then
        let page_name = if self.book_name.split(":").count() == 1 {
            // If there's no : in the name, it's simply the book's name
            &self.book_name
        } else {
            // We add the full name as callout
            children.push(json!({
                "object": "block",
                "type": "callout",
                "callout": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": self.book_name,
                            },
                        }
                    ],
                    "icon": {
                        "emoji": "📕"
                    },
                    "color": "default"
                }
            }));

            // We return the first part of the string
            self.book_name.split(":").next().unwrap()
        };

        children.push(json!({
            "object": "block",
            "type": "callout",
            "callout": {
                "rich_text": [
                    {
                        "type": "text",
                        "text": {
                            "content": self.author,
                        },
                    }
                ],
                "icon": {
                    "emoji": "✍️"
                },
                "color": "default"
            }
        }));

        children.push(json!({
            "object": "block",
            "type": "divider",
            "divider": {}
        }));

        for clip in &self.clips {
            // TODO Fix two issues: 2000 char max/block, and 100 blocks max/page
            // -> Make each quote into a row in a database???
            if clip.content.len() > 2000 {
                println!("Skipping clip because it's too long: {:?}", clip.content);
                continue;
            }

            children.push(json!({
                "object": "block",
                "type": "quote",
                "quote": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": format!("{}\n", clip.content)
                                }
                        },
                        {
                            "type": "mention",
                            "mention":{
                                "type": "date",
                                "date": {
                                    "start": clip.date,
                                }
                            }
                        }
                    }
                }
            ));

            children.push(json!({
                "object": "block",
                "type": "divider",
                "divider": {}
            }));
        }

        NotionBookPage {
            parent: json!({ "page_id": parent_page_id }),
            icon: json!({
                "emoji": "📖"
            }),
            properties: json!({
                "title": [
                        {
                            "text": {
                                "content": page_name
                            }
                        }
                    ]
            }),
            children,
        }
    }
}

pub fn upload_to_notion(
    api_key: String,
    parent_page_id: String,
    books_clips: Vec<BookClips>,
) -> Result<()> {
    let client = reqwest::blocking::Client::new();

    // TODO Cleanup if that works
    let mut headers = HeaderMap::new();
    let header_name = HeaderName::from_str(NOTION_API_VERSION_HEADER)?;
    headers.insert(header_name, NOTION_API_VERSION.parse()?);

    for book in books_clips {
        println!("Uploading clips from {:?}", book.book_name);

        let res = client
            .post(NOTION_API_URL)
            .bearer_auth(api_key.clone())
            .headers(headers.clone())
            .json(&book.to_notion_body(parent_page_id.clone()))
            .send()?;

        match res.status() {
            StatusCode::OK => {
                continue;
            }
            StatusCode::BAD_REQUEST => {
                println!("Bad request: {:?}", res.text()?);
            }
            _ => {
                println!("Unexpected status code: {:?}", res.status());
            }
        }
    }

    Ok(())
}
