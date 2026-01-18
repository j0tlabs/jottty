# Database internals for Jottty app

This document provides the schema and structure of the database used in the Jottty note-taking application.

Jottty stores all journal data directly in SQLite using a `vaults` table. The `content` column
stores Transit JSON blobs that represent entity maps DataScript-like, Datomic-like. 


## Database Internals Versions

- [version-0.1.0](./db/version-0.1.0.md) <- WIP
- [version-0.2.0](./db/version-0.2.0.md)
