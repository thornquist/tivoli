# Tivoli Server API

REST API for the Tivoli portrait photography gallery application.

## Overview

- **Base URL:** `http://localhost:3000`
- **Content-Type:** All responses are `application/json` unless otherwise noted
- **Authentication:** None (open API)

## Data Model

The API serves a photography gallery organized as:

- **Collections** (studios) contain **galleries** (shoots)
- **Galleries** contain **images**
- **Models** (people) are associated with images (many-to-many, scoped to a collection)
- **Tags** are organized into **tag groups** and associated with images (many-to-many)

All entities use UUID v4 strings as primary keys.

---

## Endpoints

### GET /collections

List all collections with image and gallery counts.

**Response:**

```typescript
Array<{
  name: string;
  image_count: number;
  gallery_count: number;
}>
```

**Example:**

```bash
curl http://localhost:3000/collections
```

```json
[
  { "name": "golden-hour-photo", "image_count": 13, "gallery_count": 3 },
  { "name": "lumiere-studio", "image_count": 14, "gallery_count": 3 },
  { "name": "noir-atelier", "image_count": 14, "gallery_count": 3 },
  { "name": "raw-collective", "image_count": 14, "gallery_count": 3 }
]
```

---

### GET /galleries

List galleries, optionally filtered by collection.

**Query Parameters:**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `collection` | string | No | Filter by collection name |

**Response:**

```typescript
Array<{
  name: string;
  collection: string;
  image_count: number;
}>
```

**Examples:**

```bash
# All galleries
curl http://localhost:3000/galleries

# Galleries in a specific collection
curl http://localhost:3000/galleries?collection=lumiere-studio
```

```json
[
  { "name": "bridal-collection", "collection": "lumiere-studio", "image_count": 5 },
  { "name": "corporate-headshots", "collection": "lumiere-studio", "image_count": 4 },
  { "name": "summer-editorial", "collection": "lumiere-studio", "image_count": 5 }
]
```

---

### GET /models

List models, optionally filtered by collection.

**Query Parameters:**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `collection` | string | No | Filter by collection name |

**Response:**

```typescript
Array<{
  uuid: string;
  name: string;
  collection: string;
}>
```

**Example:**

```bash
curl http://localhost:3000/models?collection=raw-collective
```

```json
[
  { "uuid": "a1b2c3d4-...", "name": "diego", "collection": "raw-collective" },
  { "uuid": "e5f6a7b8-...", "name": "kai", "collection": "raw-collective" },
  { "uuid": "c9d0e1f2-...", "name": "milo", "collection": "raw-collective" },
  { "uuid": "a3b4c5d6-...", "name": "nina", "collection": "raw-collective" },
  { "uuid": "e7f8a9b0-...", "name": "suki", "collection": "raw-collective" },
  { "uuid": "c1d2e3f4-...", "name": "zara", "collection": "raw-collective" }
]
```

**Note:** Models are scoped to collections. The same name may exist in different collections (they are different people).

---

### GET /tags

List all tag groups with their nested tags.

**Response:**

```typescript
Array<{
  uuid: string;
  name: string;
  tags: Array<{
    uuid: string;
    name: string;
  }>;
}>
```

**Example:**

```bash
curl http://localhost:3000/tags
```

```json
[
  {
    "uuid": "f1e2d3c4-...",
    "name": "lighting",
    "tags": [
      { "uuid": "a1b2c3d4-...", "name": "backlit" },
      { "uuid": "e5f6a7b8-...", "name": "dramatic-light" },
      { "uuid": "c9d0e1f2-...", "name": "golden-hour" },
      { "uuid": "a3b4c5d6-...", "name": "high-key" },
      { "uuid": "e7f8a9b0-...", "name": "low-key" },
      { "uuid": "c1d2e3f4-...", "name": "natural-light" },
      { "uuid": "a5b6c7d8-...", "name": "studio-light" }
    ]
  },
  {
    "uuid": "e9f0a1b2-...",
    "name": "mood",
    "tags": [
      { "uuid": "c3d4e5f6-...", "name": "bold" },
      { "uuid": "a7b8c9d0-...", "name": "dramatic" },
      { "uuid": "e1f2a3b4-...", "name": "edgy" },
      { "uuid": "c5d6e7f8-...", "name": "elegant" },
      { "uuid": "a9b0c1d2-...", "name": "moody" },
      { "uuid": "e3f4a5b6-...", "name": "playful" },
      { "uuid": "c7d8e9f0-...", "name": "romantic" },
      { "uuid": "a1b2c3d4-...", "name": "serene" }
    ]
  }
]
```

**Tag groups:** `lighting`, `setting`, `mood`, `framing`, `wardrobe`

---

### POST /images/search

Search for images using the filter DSL. Returns bare image rows for the grid view (use with lazy loading).

**Request Body:**

```typescript
{
  filters: Array<{
    field: "collection" | "gallery" | "models" | "tags";
    op: "eq" | "any_of" | "all_of" | "exact" | "none_of";
    value: string | string[];
  }>;
}
```

An empty `filters` array returns all images.

**Response:**

```typescript
Array<{
  uuid: string;
  path: string;
  collection: string;
  gallery: string;
  width: number;
  height: number;
  file_size: number;  // bytes
}>
```

**Example:**

```bash
curl -X POST http://localhost:3000/images/search \
  -H 'Content-Type: application/json' \
  -d '{
    "filters": [
      { "field": "collection", "op": "eq", "value": "lumiere-studio" },
      { "field": "gallery", "op": "eq", "value": "summer-editorial" }
    ]
  }'
```

```json
[
  {
    "uuid": "afe2f112-...",
    "path": "lumiere-studio/summer-editorial/emma-garden-bench.jpg",
    "collection": "lumiere-studio",
    "gallery": "summer-editorial",
    "width": 1920,
    "height": 1280,
    "file_size": 52481
  }
]
```

See the [Filter DSL Reference](#filter-dsl-reference) below for full details.

---

### POST /images/search/options

Get available filter options for the current filter state. Used by the filter view to show which values are available and how many images match.

**Request Body:** Same as `/images/search`.

**Response:**

```typescript
{
  image_count: number;
  collections: string[];
  galleries: Array<{ name: string; collection: string; image_count: number }>;
  models: Array<{ uuid: string; name: string; collection: string }>;
  tags: Array<{ uuid: string; name: string; group: string }>;
}
```

**Example:**

```bash
curl -X POST http://localhost:3000/images/search/options \
  -H 'Content-Type: application/json' \
  -d '{
    "filters": [
      { "field": "collection", "op": "eq", "value": "lumiere-studio" }
    ]
  }'
```

```json
{
  "image_count": 14,
  "collections": ["lumiere-studio"],
  "galleries": [
    { "name": "bridal-collection", "collection": "lumiere-studio", "image_count": 0 },
    { "name": "corporate-headshots", "collection": "lumiere-studio", "image_count": 0 },
    { "name": "summer-editorial", "collection": "lumiere-studio", "image_count": 0 }
  ],
  "models": [
    { "uuid": "6156a42b-...", "name": "anna", "collection": "lumiere-studio" },
    { "uuid": "a1b2c3d4-...", "name": "clara", "collection": "lumiere-studio" }
  ],
  "tags": [
    { "uuid": "7b9441ae-...", "name": "natural-light", "group": "lighting" },
    { "uuid": "ca5de307-...", "name": "outdoor", "group": "setting" }
  ]
}
```

---

### GET /images/{uuid}

Get full image details including associated models and tags.

**Path Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `uuid` | string | Image UUID |

**Response:**

```typescript
{
  uuid: string;
  path: string;
  collection: string;
  gallery: string;
  width: number;
  height: number;
  file_size: number;  // bytes
  models: Array<{ uuid: string; name: string; collection: string }>;
  tags: Array<{ uuid: string; name: string; group: string }>;
}
```

**Example:**

```bash
curl http://localhost:3000/images/afe2f112-2c82-4de4-bc91-cc82b1739eda
```

---

### GET /images/{uuid}/file

Serve the actual image file.

**Path Parameters:**

| Parameter | Type | Description |
|---|---|---|
| `uuid` | string | Image UUID |

**Response:**

- **200 OK** — JPEG image bytes with `Content-Type: image/jpeg`
- **404 Not Found** — Image UUID not in database or file missing from disk

**Example:**

```bash
curl -o photo.jpg http://localhost:3000/images/afe2f112-2c82-4de4-bc91-cc82b1739eda/file
```

---

## Filter DSL Reference

The filter DSL is used with `POST /images/search`. Filters are expressed as an array of clauses, all of which are AND'd together.

### Fields and Allowed Operators

| Field | `eq` | `any_of` | `all_of` | `exact` | `none_of` |
|---|---|---|---|---|---|
| `collection` | Yes | - | - | - | - |
| `gallery` | Yes | - | - | - | - |
| `models` | - | Yes | Yes | Yes | Yes |
| `tags` | - | Yes | Yes | - | Yes |

Using an unsupported operator for a field returns **400 Bad Request**.

### Operator Semantics

#### `eq`
Exact string match. Used for `collection` and `gallery` fields. Takes a single string value.

```json
{ "field": "collection", "op": "eq", "value": "lumiere-studio" }
```

#### `any_of`
Image must be associated with **at least one** of the provided UUIDs. Takes an array of UUIDs.

```json
{ "field": "models", "op": "any_of", "value": ["uuid1", "uuid2"] }
```

#### `all_of`
Image must be associated with **all** of the provided UUIDs (and may have additional ones). Takes an array of UUIDs.

```json
{ "field": "tags", "op": "all_of", "value": ["uuid-outdoor", "uuid-casual"] }
```

#### `exact`
Image must be associated with **exactly** the provided UUIDs — no more, no less. Only available for `models`. Takes an array of UUIDs.

```json
{ "field": "models", "op": "exact", "value": ["uuid-emma"] }
```

#### `none_of`
Image must **not** be associated with any of the provided UUIDs. Takes an array of UUIDs.

```json
{ "field": "tags", "op": "none_of", "value": ["uuid-studio"] }
```

### Multiple Clauses

All clauses are AND'd together. You can have multiple clauses on the same field:

```json
{
  "filters": [
    { "field": "tags", "op": "any_of", "value": ["uuid-natural-light", "uuid-golden-hour"] },
    { "field": "tags", "op": "all_of", "value": ["uuid-outdoor", "uuid-casual"] }
  ]
}
```

This matches images that have **(natural-light OR golden-hour) AND (outdoor AND casual)**.

### Common Query Patterns

#### Find all images of a specific model

```json
{
  "filters": [
    { "field": "models", "op": "any_of", "value": ["<model-uuid>"] }
  ]
}
```

#### Find solo shots only (exactly one specific model, no one else)

```json
{
  "filters": [
    { "field": "models", "op": "exact", "value": ["<model-uuid>"] }
  ]
}
```

#### Find duo shots (images with both models)

```json
{
  "filters": [
    { "field": "models", "op": "all_of", "value": ["<model-uuid-1>", "<model-uuid-2>"] }
  ]
}
```

#### Find images with warm lighting

```json
{
  "filters": [
    { "field": "tags", "op": "any_of", "value": ["<uuid-golden-hour>", "<uuid-natural-light>", "<uuid-backlit>"] }
  ]
}
```

#### Find outdoor casual images excluding a specific model

```json
{
  "filters": [
    { "field": "tags", "op": "all_of", "value": ["<uuid-outdoor>", "<uuid-casual>"] },
    { "field": "models", "op": "none_of", "value": ["<model-uuid>"] }
  ]
}
```

#### Browse a specific studio's shoot

```json
{
  "filters": [
    { "field": "collection", "op": "eq", "value": "noir-atelier" },
    { "field": "gallery", "op": "eq", "value": "film-noir" }
  ]
}
```

---

## Error Responses

All errors return JSON with an `error` field:

```json
{ "error": "description of what went wrong" }
```

| Status Code | Meaning |
|---|---|
| 400 | Bad request — invalid filter operator, missing required value |
| 404 | Not found — image UUID does not exist |
| 422 | Unprocessable entity — malformed JSON body |
| 500 | Internal server error — database or server failure |

---

## Configuration

The server accepts these environment variables:

| Variable | Default | Description |
|---|---|---|
| `PORT` | `3000` | Port to listen on |
| `TIVOLI_DB_PATH` | `../data/tivoli.db` | Path to SQLite database |
| `TIVOLI_GALLERIES_PATH` | `../galleries` | Path to image files directory |
