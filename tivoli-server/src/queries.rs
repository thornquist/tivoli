use std::collections::HashMap;

use crate::errors::AppError;
use crate::models::*;

// --- Filter DSL query builder ---

pub fn build_image_query(filters: &[FilterClause]) -> Result<(String, Vec<String>), AppError> {
    let mut conditions: Vec<String> = Vec::new();
    let mut params: Vec<String> = Vec::new();

    for clause in filters {
        validate_clause(clause)?;
        match clause.field {
            FilterField::Collection => {
                let val = clause.value.as_single().ok_or_else(|| {
                    AppError::BadRequest("collection eq requires a single value".into())
                })?;
                conditions.push("i.collection = ?".into());
                params.push(val.to_string());
            }
            FilterField::Gallery => {
                let val = clause.value.as_single().ok_or_else(|| {
                    AppError::BadRequest("gallery eq requires a single value".into())
                })?;
                conditions.push("i.gallery = ?".into());
                params.push(val.to_string());
            }
            FilterField::Models => {
                let vals = clause.value.as_multiple();
                let placeholders = make_placeholders(vals.len());
                match clause.op {
                    FilterOp::AnyOf => {
                        conditions.push(format!(
                            "i.uuid IN (SELECT image_uuid FROM image_models WHERE model_uuid IN ({placeholders}))"
                        ));
                    }
                    FilterOp::AllOf => {
                        conditions.push(format!(
                            "i.uuid IN (SELECT image_uuid FROM image_models WHERE model_uuid IN ({placeholders}) GROUP BY image_uuid HAVING COUNT(DISTINCT model_uuid) = {})",
                            vals.len()
                        ));
                    }
                    FilterOp::Exact => {
                        conditions.push(format!(
                            "i.uuid IN (SELECT image_uuid FROM image_models WHERE model_uuid IN ({placeholders}) GROUP BY image_uuid HAVING COUNT(DISTINCT model_uuid) = {})",
                            vals.len()
                        ));
                        params.extend(vals.iter().map(|v| v.to_string()));
                        let placeholders2 = make_placeholders(vals.len());
                        conditions.push(format!(
                            "i.uuid NOT IN (SELECT image_uuid FROM image_models WHERE model_uuid NOT IN ({placeholders2}))"
                        ));
                    }
                    FilterOp::NoneOf => {
                        conditions.push(format!(
                            "i.uuid NOT IN (SELECT image_uuid FROM image_models WHERE model_uuid IN ({placeholders}))"
                        ));
                    }
                    _ => unreachable!(),
                }
                if clause.op != FilterOp::Exact {
                    params.extend(vals.iter().map(|v| v.to_string()));
                } else {
                    params.extend(vals.iter().map(|v| v.to_string()));
                }
            }
            FilterField::Tags => {
                let vals = clause.value.as_multiple();
                let placeholders = make_placeholders(vals.len());
                match clause.op {
                    FilterOp::AnyOf => {
                        conditions.push(format!(
                            "i.uuid IN (SELECT image_uuid FROM image_tags WHERE tag_uuid IN ({placeholders}))"
                        ));
                    }
                    FilterOp::AllOf => {
                        conditions.push(format!(
                            "i.uuid IN (SELECT image_uuid FROM image_tags WHERE tag_uuid IN ({placeholders}) GROUP BY image_uuid HAVING COUNT(DISTINCT tag_uuid) = {})",
                            vals.len()
                        ));
                    }
                    FilterOp::NoneOf => {
                        conditions.push(format!(
                            "i.uuid NOT IN (SELECT image_uuid FROM image_tags WHERE tag_uuid IN ({placeholders}))"
                        ));
                    }
                    _ => unreachable!(),
                }
                params.extend(vals.iter().map(|v| v.to_string()));
            }
        }
    }

    let mut sql = "SELECT i.uuid, i.path, i.collection, i.gallery FROM images i".to_string();
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }
    sql.push_str(" ORDER BY i.collection, i.gallery, i.path");

    Ok((sql, params))
}

fn validate_clause(clause: &FilterClause) -> Result<(), AppError> {
    match (&clause.field, &clause.op) {
        (FilterField::Collection | FilterField::Gallery, op) if *op != FilterOp::Eq => {
            Err(AppError::BadRequest(format!(
                "{} only supports the 'eq' operator",
                if clause.field == FilterField::Collection { "collection" } else { "gallery" }
            )))
        }
        (FilterField::Tags, FilterOp::Exact) => {
            Err(AppError::BadRequest("tags does not support the 'exact' operator".into()))
        }
        (FilterField::Tags | FilterField::Models, FilterOp::Eq) => {
            Err(AppError::BadRequest(format!(
                "{} does not support the 'eq' operator",
                if clause.field == FilterField::Tags { "tags" } else { "models" }
            )))
        }
        _ => Ok(()),
    }
}

fn make_placeholders(n: usize) -> String {
    let v: Vec<&str> = (0..n).map(|_| "?").collect();
    v.join(", ")
}

// --- Query functions ---

pub fn query_images(
    conn: &rusqlite::Connection,
    sql: &str,
    params: &[String],
) -> Result<Vec<ImageRow>, rusqlite::Error> {
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(ImageRow {
            uuid: row.get(0)?,
            path: row.get(1)?,
            collection: row.get(2)?,
            gallery: row.get(3)?,
        })
    })?;
    rows.collect()
}

pub fn models_for_images(
    conn: &rusqlite::Connection,
    image_uuids: &[&str],
) -> Result<HashMap<String, Vec<ModelRef>>, rusqlite::Error> {
    if image_uuids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = make_placeholders(image_uuids.len());
    let sql = format!(
        "SELECT im.image_uuid, m.uuid, m.name FROM image_models im JOIN models m ON im.model_uuid = m.uuid WHERE im.image_uuid IN ({placeholders})"
    );
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        image_uuids.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&sql)?;
    let mut map: HashMap<String, Vec<ModelRef>> = HashMap::new();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            ModelRef {
                uuid: row.get(1)?,
                name: row.get(2)?,
            },
        ))
    })?;
    for row in rows {
        let (image_uuid, model_ref) = row?;
        map.entry(image_uuid).or_default().push(model_ref);
    }
    Ok(map)
}

pub fn tags_for_images(
    conn: &rusqlite::Connection,
    image_uuids: &[&str],
) -> Result<HashMap<String, Vec<TagRef>>, rusqlite::Error> {
    if image_uuids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = make_placeholders(image_uuids.len());
    let sql = format!(
        "SELECT it.image_uuid, t.uuid, t.name, tg.name FROM image_tags it JOIN tags t ON it.tag_uuid = t.uuid JOIN tag_groups tg ON t.tag_group_uuid = tg.uuid WHERE it.image_uuid IN ({placeholders})"
    );
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        image_uuids.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(&sql)?;
    let mut map: HashMap<String, Vec<TagRef>> = HashMap::new();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok((
            row.get::<_, String>(0)?,
            TagRef {
                uuid: row.get(1)?,
                name: row.get(2)?,
                group: row.get(3)?,
            },
        ))
    })?;
    for row in rows {
        let (image_uuid, tag_ref) = row?;
        map.entry(image_uuid).or_default().push(tag_ref);
    }
    Ok(map)
}

pub fn query_collections(
    conn: &rusqlite::Connection,
) -> Result<Vec<CollectionSummary>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT i.collection, COUNT(*) as image_count, COUNT(DISTINCT i.gallery) as gallery_count FROM images i GROUP BY i.collection ORDER BY i.collection",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(CollectionSummary {
            name: row.get(0)?,
            image_count: row.get(1)?,
            gallery_count: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn query_galleries(
    conn: &rusqlite::Connection,
    collection: Option<&str>,
) -> Result<Vec<GallerySummary>, rusqlite::Error> {
    let (sql, params_owned);
    if let Some(c) = collection {
        sql = "SELECT i.gallery, i.collection, COUNT(*) as image_count FROM images i WHERE i.collection = ? GROUP BY i.collection, i.gallery ORDER BY i.collection, i.gallery";
        params_owned = vec![c.to_string()];
    } else {
        sql = "SELECT i.gallery, i.collection, COUNT(*) as image_count FROM images i GROUP BY i.collection, i.gallery ORDER BY i.collection, i.gallery";
        params_owned = vec![];
    }
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_owned.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(GallerySummary {
            name: row.get(0)?,
            collection: row.get(1)?,
            image_count: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn query_models(
    conn: &rusqlite::Connection,
    collection: Option<&str>,
) -> Result<Vec<Model>, rusqlite::Error> {
    let (sql, params_owned);
    if let Some(c) = collection {
        sql = "SELECT uuid, name, collection FROM models WHERE collection = ? ORDER BY name";
        params_owned = vec![c.to_string()];
    } else {
        sql = "SELECT uuid, name, collection FROM models ORDER BY collection, name";
        params_owned = vec![];
    }
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params_owned.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(Model {
            uuid: row.get(0)?,
            name: row.get(1)?,
            collection: row.get(2)?,
        })
    })?;
    rows.collect()
}

pub fn query_tag_groups(
    conn: &rusqlite::Connection,
) -> Result<Vec<TagGroup>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT tg.uuid, tg.name, t.uuid, t.name FROM tag_groups tg LEFT JOIN tags t ON t.tag_group_uuid = tg.uuid ORDER BY tg.name, t.name",
    )?;
    let mut groups: Vec<TagGroup> = Vec::new();
    let mut current_group: Option<TagGroup> = None;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;

    for row in rows {
        let (group_uuid, group_name, tag_uuid, tag_name) = row?;
        match &mut current_group {
            Some(g) if g.uuid == group_uuid => {
                if let (Some(tu), Some(tn)) = (tag_uuid, tag_name) {
                    g.tags.push(Tag { uuid: tu, name: tn });
                }
            }
            _ => {
                if let Some(g) = current_group.take() {
                    groups.push(g);
                }
                let mut tags = Vec::new();
                if let (Some(tu), Some(tn)) = (tag_uuid, tag_name) {
                    tags.push(Tag { uuid: tu, name: tn });
                }
                current_group = Some(TagGroup {
                    uuid: group_uuid,
                    name: group_name,
                    tags,
                });
            }
        }
    }
    if let Some(g) = current_group {
        groups.push(g);
    }
    Ok(groups)
}
