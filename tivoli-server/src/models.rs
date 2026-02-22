use serde::{Deserialize, Serialize};

// --- Filter DSL types ---

#[derive(Deserialize)]
pub struct SearchRequest {
    pub filters: Vec<FilterClause>,
}

#[derive(Deserialize)]
pub struct FilterClause {
    pub field: FilterField,
    pub op: FilterOp,
    pub value: FilterValue,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterField {
    Collection,
    Gallery,
    Models,
    Tags,
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FilterOp {
    Eq,
    AnyOf,
    AllOf,
    Exact,
    NoneOf,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Single(String),
    Multiple(Vec<String>),
}

impl FilterValue {
    pub fn as_single(&self) -> Option<&str> {
        match self {
            FilterValue::Single(s) => Some(s),
            FilterValue::Multiple(v) if v.len() == 1 => Some(&v[0]),
            _ => None,
        }
    }

    pub fn as_multiple(&self) -> Vec<&str> {
        match self {
            FilterValue::Single(s) => vec![s.as_str()],
            FilterValue::Multiple(v) => v.iter().map(|s| s.as_str()).collect(),
        }
    }
}

// --- Tag mutation ---

#[derive(Deserialize)]
pub struct UpdateTagsRequest {
    pub tag_uuids: Vec<String>,
}

// --- Query parameter structs ---

#[derive(Deserialize)]
pub struct CollectionFilter {
    pub collection: Option<String>,
}

#[derive(Deserialize)]
pub struct ImageFileParams {
    pub w: Option<u32>,
}

// --- Response structs ---

#[derive(Serialize)]
pub struct FilterOptions {
    pub image_count: u32,
    pub collections: Vec<String>,
    pub galleries: Vec<GallerySummary>,
    pub models: Vec<Model>,
    pub tags: Vec<TagRef>,
}

#[derive(Serialize, Clone)]
pub struct TagRef {
    pub uuid: String,
    pub name: String,
    pub group: String,
}

#[derive(Serialize)]
pub struct CollectionSummary {
    pub name: String,
    pub image_count: u32,
    pub gallery_count: u32,
}

#[derive(Serialize)]
pub struct GallerySummary {
    pub name: String,
    pub collection: String,
    pub image_count: u32,
}

#[derive(Serialize)]
pub struct Model {
    pub uuid: String,
    pub name: String,
    pub collection: String,
}

#[derive(Serialize)]
pub struct TagGroup {
    pub uuid: String,
    pub name: String,
    pub tags: Vec<Tag>,
}

#[derive(Serialize)]
pub struct Tag {
    pub uuid: String,
    pub name: String,
}

// --- Internal types ---

#[derive(Serialize)]
pub struct ImageRow {
    pub uuid: String,
    pub path: String,
    pub collection: String,
    pub gallery: String,
    pub width: u32,
    pub height: u32,
    pub file_size: i64,
}

#[derive(Serialize)]
pub struct ImageDetail {
    pub uuid: String,
    pub path: String,
    pub collection: String,
    pub gallery: String,
    pub width: u32,
    pub height: u32,
    pub file_size: i64,
    pub models: Vec<Model>,
    pub tags: Vec<TagRef>,
}
