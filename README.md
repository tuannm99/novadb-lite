# [#2] sqlite

## Scopes

- Single file storage, limited to 5Gb.
- SlottedPage Layout
- Simple CRUD operations on TABLE
- Index-Organize Storage (Cluster idx - btree int only)
- OverflowPage
- Simple Buffer
- Global lock - database level lock only -- can extends if can
- Wal for support durability, simple transaction
- Vietnamese comment
