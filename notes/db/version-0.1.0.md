# Notes App - Database Schema v0.1.0

### Table (Vaults) - source of truth
```sql
CREATE TABLE IF NOT EXISTS vaults (
  addr INTEGER PRIMARY KEY,    -- Address/pointer identifier
  content TEXT,                 -- Transit JSON blob (entity map)
  addresses JSON                -- Array of referenced addresses
)
```
addr: pointer used by the app to identify data chunks.
content: serialized Transit JSON representing a single entity map.
addresses: JSON array of other addr values that this address references (for GC / traversal).

### Entity map format (Transit JSON)
Each entity is stored as a map with an id and attributes.

```json
{
  "id": "block:2026-01-10-1700000000000000000",
  "attrs": {
    "block/title": "January 10, 2026",
    "block/content": "TODO: Finish the project",
    "block/page": "page:2026-01-10"
  }
}
```

A journal page is also an entity:

```json
{
  "id": "page:2026-01-10",
  "attrs": {
    "page/name": "2026-01-10"
  }
}
```

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

