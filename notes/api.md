# Notes API 

This document describes the Notes App API for storing and querying notes data.
This will be available Version 0.2.0



### Transaction format
The API accepts datoms in list form:

```json
[
  ["db/add", "block:2026-01-10-1700000000000000000", "block/title", "January 10, 2026"],
  ["db/add", "block:2026-01-10-1700000000000000000", "block/content", "TODO: Finish the project"],
  ["db/add", "block:2026-01-10-1700000000000000000", "block/page", "page:2026-01-10"],
  ["db/add", "page:2026-01-10", "page/name", "2026-01-10"]
]
```
## API
if you POST taht to POST `/api/transact`
```json
{
  "datoms": [
    ["db/add", "block:2026-01-10-1700000000000000000", "block/title", "January 10, 2026"],
    ["db/add", "block:2026-01-10-1700000000000000000", "block/content", "TODO: Finish the project"],
    ["db/add", "block:2026-01-10-1700000000000000000", "block/page", "page:2026-01-10"],
    ["db/add", "page:2026-01-10", "page/name", "2026-01-10"]
  ]
}

```

Response:

```json
{
  "entities": [
        { "id": "block:2026-01-10-1700000000000000000",
            "attrs": {
                "block/title": "January 10, 2026",
                "block/content": "TODO: Finish the project",
                "block/page": "page:2026-01-10"
            }
        }
    ]
}
```

The query api
`POST /query`

```json
{ "kind": "pages" }
```

```json
{ "kind": "page_blocks", "page": "page:2026-01-10" }
```

```json
{ "kind": "search", "term": "TODO" }
```

```json
{ "kind": "search", "term": "tag:TODO" }
```

