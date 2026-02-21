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
                    FilterOp::Exact => {
                        // Must have all selected tags
                        conditions.push(format!(
                            "i.uuid IN (SELECT image_uuid FROM image_tags WHERE tag_uuid IN ({placeholders}) GROUP BY image_uuid HAVING COUNT(DISTINCT tag_uuid) = {})",
                            vals.len()
                        ));
                        params.extend(vals.iter().map(|v| v.to_string()));
                        // Must not have other tags from the same group(s)
                        let placeholders2 = make_placeholders(vals.len());
                        conditions.push(format!(
                            "i.uuid NOT IN (SELECT it2.image_uuid FROM image_tags it2 JOIN tags t2 ON it2.tag_uuid = t2.uuid WHERE t2.tag_group_uuid IN (SELECT tag_group_uuid FROM tags WHERE uuid IN ({placeholders2})) AND it2.tag_uuid NOT IN ({placeholders2}))"
                        ));
                        params.extend(vals.iter().map(|v| v.to_string()));
                        params.extend(vals.iter().map(|v| v.to_string()));
                    }
                    FilterOp::NoneOf => {
                        conditions.push(format!(
                            "i.uuid NOT IN (SELECT image_uuid FROM image_tags WHERE tag_uuid IN ({placeholders}))"
                        ));
                    }
                    _ => unreachable!(),
                }
                // For exact, params are already added inline above
                if clause.op == FilterOp::Exact {
                    // already handled
                } else {
                    params.extend(vals.iter().map(|v| v.to_string()));
                }
            }
        }
    }

    let mut sql = "SELECT i.uuid, i.path, i.collection, i.gallery, i.width, i.height FROM images i".to_string();
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
        (FilterField::Tags, FilterOp::Exact) => Ok(()),
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
            width: row.get(4)?,
            height: row.get(5)?,
        })
    })?;
    rows.collect()
}

pub fn query_filter_options(
    conn: &rusqlite::Connection,
    filters: &[FilterClause],
) -> Result<FilterOptions, AppError> {
    let (image_sql, params) = build_image_query(filters)?;
    // Use the image query as a subquery (strip the ORDER BY for efficiency)
    let subquery = image_sql.replace(" ORDER BY i.collection, i.gallery, i.path", "");
    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        params.iter().map(|s| s as &dyn rusqlite::types::ToSql).collect();

    // Count
    let count_sql = format!("SELECT COUNT(*) FROM ({subquery})");
    let image_count: u32 = conn
        .query_row(&count_sql, param_refs.as_slice(), |row| row.get(0))?;

    // Distinct collections
    let coll_sql = format!(
        "SELECT DISTINCT collection FROM ({subquery}) ORDER BY collection",
    );
    let mut stmt = conn.prepare(&coll_sql)?;
    let collections: Vec<String> = stmt
        .query_map(param_refs.as_slice(), |row| row.get(0))?
        .collect::<Result<_, _>>()?;

    // Distinct galleries
    let gal_sql = format!(
        "SELECT DISTINCT collection, gallery FROM ({subquery}) ORDER BY collection, gallery",
    );
    let mut stmt = conn.prepare(&gal_sql)?;
    let galleries: Vec<GallerySummary> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(GallerySummary {
                collection: row.get(0)?,
                name: row.get(1)?,
                image_count: 0,
            })
        })?
        .collect::<Result<_, _>>()?;

    // Distinct models
    let model_sql = format!(
        "SELECT DISTINCT m.uuid, m.name, m.collection \
         FROM image_models im \
         JOIN models m ON im.model_uuid = m.uuid \
         WHERE im.image_uuid IN (SELECT uuid FROM ({subquery})) \
         ORDER BY m.collection, m.name",
    );
    let mut stmt = conn.prepare(&model_sql)?;
    let models: Vec<Model> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(Model {
                uuid: row.get(0)?,
                name: row.get(1)?,
                collection: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    // Distinct tags
    let tag_sql = format!(
        "SELECT DISTINCT t.uuid, t.name, tg.name \
         FROM image_tags it \
         JOIN tags t ON it.tag_uuid = t.uuid \
         JOIN tag_groups tg ON t.tag_group_uuid = tg.uuid \
         WHERE it.image_uuid IN (SELECT uuid FROM ({subquery})) \
         ORDER BY tg.name, t.name",
    );
    let mut stmt = conn.prepare(&tag_sql)?;
    let tags: Vec<TagRef> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(TagRef {
                uuid: row.get(0)?,
                name: row.get(1)?,
                group: row.get(2)?,
            })
        })?
        .collect::<Result<_, _>>()?;

    Ok(FilterOptions {
        image_count,
        collections,
        galleries,
        models,
        tags,
    })
}

pub fn replace_image_tags(
    conn: &rusqlite::Connection,
    image_uuid: &str,
    tag_uuids: &[String],
) -> Result<(), AppError> {
    // Verify image exists
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM images WHERE uuid = ?)",
            [image_uuid],
            |row| row.get(0),
        )
        .map_err(|e| AppError::DbError(e.to_string()))?;
    if !exists {
        return Err(AppError::NotFound("Image not found".into()));
    }

    // Verify all tag UUIDs exist
    for tag_uuid in tag_uuids {
        let tag_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM tags WHERE uuid = ?)",
                [tag_uuid],
                |row| row.get(0),
            )
            .map_err(|e| AppError::DbError(e.to_string()))?;
        if !tag_exists {
            return Err(AppError::BadRequest(format!(
                "Tag not found: {tag_uuid}"
            )));
        }
    }

    conn.execute("DELETE FROM image_tags WHERE image_uuid = ?", [image_uuid])?;
    let mut stmt =
        conn.prepare("INSERT INTO image_tags (image_uuid, tag_uuid) VALUES (?, ?)")?;
    for tag_uuid in tag_uuids {
        stmt.execute(rusqlite::params![image_uuid, tag_uuid])?;
    }
    Ok(())
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
