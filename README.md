# Kindle Clippings to Notion

Small Rust program to parse Kindle clips and upload them to Notion.

## Env variables

```ini
NOTION_API_KEY=...
NOTION_PAGE_ID=...
```

## Usage

- Create a Notion API "Internal integration" at <https://www.notion.so/my-integrations> with read, update, and insert content capabilities
- Download and save the [latest release binary from the releases page](https://github.com/mrtolkien/kindle_to_notion/releases) to the root of your Kindle
- Create a `.env` file at the root of your Kindle with the same structure as [this example `.env` file](https://github.com/mrtolkien/kindle_to_notion/blob/main/.env.example)
  - `NOTION_API_KEY` is the API key for your integration
  - `NOTION_PAGE_ID` is the page ID of the page where you want to insert your clippings pages
- Run the executable from the root of your Kindle and see it populate

## Behaviour and limitations

- TODO
## Limitations

<!-- TODO REMOVE DIVIDER BLOCK!!! -->
<!-- TODO Add releases + how to use! -->
- There can only be 100 blocks in a Notion page, so you need to have fewer quotes than this per book
- Single blocks cannot be more than 2000 characters long, so longer quotes get split in multiple blocks
