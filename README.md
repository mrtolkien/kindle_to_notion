# Kindle Clippings to Notion

Small Rust program to parse Kindle clips and upload them to Notion.

## Env variables

```ini
NOTION_API_KEY=...
NOTION_PARENT_PAGE_NAME=...
```

## Limitations

<!-- TODO REMOVE DIVIDER BLOCK!!! -->
- There can only be 100 blocks in a Notion page, so you need to have fewer quotes than this per book
- Single blocks cannot be more than 2000 characters long, so longer quotes get split in multiple blocks
