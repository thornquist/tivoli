use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::oneshot;

/// Spawn the app on a random port, return base URL.
async fn spawn_app() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{port}");

    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        let app = tivoli_server::build_app("../data/tivoli.db", "../galleries");
        tx.send(()).unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Wait for the spawned task to signal it's ready
    rx.await.unwrap();
    base_url
}

async fn search(client: &Client, base: &str, filters: Value) -> Value {
    client
        .post(&format!("{base}/images/search"))
        .json(&json!({ "filters": filters }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn search_count(client: &Client, base: &str, filters: Value) -> usize {
    let v = search(client, base, filters).await;
    v.as_array().unwrap().len()
}

async fn search_options(client: &Client, base: &str, filters: Value) -> Value {
    client
        .post(&format!("{base}/images/search/options"))
        .json(&json!({ "filters": filters }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

// ─── GET /collections ───

#[tokio::test]
async fn test_collections_returns_all_four() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 4);
    let names: Vec<&str> = resp.iter().map(|c| c["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"lumiere-studio"));
    assert!(names.contains(&"raw-collective"));
    assert!(names.contains(&"golden-hour-photo"));
    assert!(names.contains(&"noir-atelier"));
}

#[tokio::test]
async fn test_collections_counts_match() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/collections"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    for col in &resp {
        let name = col["name"].as_str().unwrap();
        let image_count = col["image_count"].as_u64().unwrap();
        let gallery_count = col["gallery_count"].as_u64().unwrap();
        match name {
            "lumiere-studio" => {
                assert_eq!(image_count, 14);
                assert_eq!(gallery_count, 3);
            }
            "raw-collective" => {
                assert_eq!(image_count, 14);
                assert_eq!(gallery_count, 3);
            }
            "golden-hour-photo" => {
                assert_eq!(image_count, 13);
                assert_eq!(gallery_count, 3);
            }
            "noir-atelier" => {
                assert_eq!(image_count, 14);
                assert_eq!(gallery_count, 3);
            }
            _ => panic!("Unexpected collection: {name}"),
        }
    }
}

// ─── GET /galleries ───

#[tokio::test]
async fn test_galleries_unfiltered() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/galleries"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 12);
}

#[tokio::test]
async fn test_galleries_filtered_by_collection() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/galleries?collection=lumiere-studio"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 3);
    for g in &resp {
        assert_eq!(g["collection"].as_str().unwrap(), "lumiere-studio");
    }
}

#[tokio::test]
async fn test_galleries_unknown_collection() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/galleries?collection=nonexistent"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 0);
}

// ─── GET /models ───

#[tokio::test]
async fn test_models_unfiltered() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/models"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 25);
}

#[tokio::test]
async fn test_models_filtered_by_collection() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/models?collection=raw-collective"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 6);
    let names: Vec<&str> = resp.iter().map(|m| m["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"kai"));
    assert!(names.contains(&"zara"));
    assert!(names.contains(&"milo"));
    assert!(names.contains(&"nina"));
    assert!(names.contains(&"diego"));
    assert!(names.contains(&"suki"));
}

// ─── GET /tags ───

#[tokio::test]
async fn test_tags_returns_five_groups() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/tags"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp.len(), 5);
}

#[tokio::test]
async fn test_tags_groups_are_named() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/tags"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let names: Vec<&str> = resp.iter().map(|g| g["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"lighting"));
    assert!(names.contains(&"setting"));
    assert!(names.contains(&"mood"));
    assert!(names.contains(&"framing"));
    assert!(names.contains(&"wardrobe"));
}

#[tokio::test]
async fn test_tags_nested_structure() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp: Vec<Value> = client
        .get(&format!("{base}/tags"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let total_tags: usize = resp
        .iter()
        .map(|g| g["tags"].as_array().unwrap().len())
        .sum();
    assert_eq!(total_tags, 34);
}

// ─── POST /images/search — basic ───

#[tokio::test]
async fn test_search_no_filters() {
    let base = spawn_app().await;
    let client = Client::new();
    assert_eq!(search_count(&client, &base, json!([])).await, 55);
}

#[tokio::test]
async fn test_search_collection_eq() {
    let base = spawn_app().await;
    let client = Client::new();
    let count = search_count(
        &client,
        &base,
        json!([{"field": "collection", "op": "eq", "value": "noir-atelier"}]),
    )
    .await;
    assert_eq!(count, 14);
}

#[tokio::test]
async fn test_search_gallery_eq() {
    let base = spawn_app().await;
    let client = Client::new();
    let count = search_count(
        &client,
        &base,
        json!([{"field": "gallery", "op": "eq", "value": "summer-editorial"}]),
    )
    .await;
    assert_eq!(count, 5);
}

// ─── POST /images/search — model filters ───

/// Helper: get model UUID by name and collection
async fn get_model_uuid(client: &Client, base: &str, name: &str, collection: &str) -> String {
    let resp: Vec<Value> = client
        .get(&format!("{base}/models?collection={collection}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    resp.iter()
        .find(|m| m["name"].as_str().unwrap() == name)
        .unwrap()["uuid"]
        .as_str()
        .unwrap()
        .to_string()
}

/// Helper: get tag UUID by name
async fn get_tag_uuid(client: &Client, base: &str, tag_name: &str) -> String {
    let resp: Vec<Value> = client
        .get(&format!("{base}/tags"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    for group in &resp {
        for tag in group["tags"].as_array().unwrap() {
            if tag["name"].as_str().unwrap() == tag_name {
                return tag["uuid"].as_str().unwrap().to_string();
            }
        }
    }
    panic!("Tag not found: {tag_name}");
}

#[tokio::test]
async fn test_search_models_any_of() {
    let base = spawn_app().await;
    let client = Client::new();
    let emma = get_model_uuid(&client, &base, "emma", "lumiere-studio").await;
    let sofia = get_model_uuid(&client, &base, "sofia", "lumiere-studio").await;

    let results = search(
        &client,
        &base,
        json!([{"field": "models", "op": "any_of", "value": [emma, sofia]}]),
    )
    .await;
    let arr = results.as_array().unwrap();
    // emma: 3 images, sofia: 3 images, duo: 1 shared → 5 unique
    assert_eq!(arr.len(), 5);
}

#[tokio::test]
async fn test_search_models_all_of() {
    let base = spawn_app().await;
    let client = Client::new();
    let emma = get_model_uuid(&client, &base, "emma", "lumiere-studio").await;
    let sofia = get_model_uuid(&client, &base, "sofia", "lumiere-studio").await;

    let count = search_count(
        &client,
        &base,
        json!([{"field": "models", "op": "all_of", "value": [emma, sofia]}]),
    )
    .await;
    // Only the duo shot has both
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_search_models_exact() {
    let base = spawn_app().await;
    let client = Client::new();
    let emma = get_model_uuid(&client, &base, "emma", "lumiere-studio").await;

    let count = search_count(
        &client,
        &base,
        json!([{"field": "models", "op": "exact", "value": [emma]}]),
    )
    .await;
    // emma solo shots only (excludes duo with sofia)
    assert_eq!(count, 2);
}

#[tokio::test]
async fn test_search_models_none_of() {
    let base = spawn_app().await;
    let client = Client::new();
    let emma = get_model_uuid(&client, &base, "emma", "lumiere-studio").await;

    // All lumiere-studio images minus those with emma
    let count = search_count(
        &client,
        &base,
        json!([
            {"field": "collection", "op": "eq", "value": "lumiere-studio"},
            {"field": "models", "op": "none_of", "value": [emma]}
        ]),
    )
    .await;
    // 14 total - 3 with emma = 11
    assert_eq!(count, 11);
}

// ─── POST /images/search — tag filters ───

#[tokio::test]
async fn test_search_tags_any_of() {
    let base = spawn_app().await;
    let client = Client::new();
    let golden = get_tag_uuid(&client, &base, "golden-hour").await;
    let backlit = get_tag_uuid(&client, &base, "backlit").await;

    let count = search_count(
        &client,
        &base,
        json!([{"field": "tags", "op": "any_of", "value": [golden, backlit]}]),
    )
    .await;
    // golden-hour: 8 images, backlit: 2 images, some overlap possible
    assert!(count >= 2 && count <= 10);
}

#[tokio::test]
async fn test_search_tags_all_of() {
    let base = spawn_app().await;
    let client = Client::new();
    let outdoor = get_tag_uuid(&client, &base, "outdoor").await;
    let casual = get_tag_uuid(&client, &base, "casual").await;

    let count = search_count(
        &client,
        &base,
        json!([{"field": "tags", "op": "all_of", "value": [outdoor, casual]}]),
    )
    .await;
    assert!(count > 0);
}

#[tokio::test]
async fn test_search_tags_none_of() {
    let base = spawn_app().await;
    let client = Client::new();
    let studio = get_tag_uuid(&client, &base, "studio").await;

    let count = search_count(
        &client,
        &base,
        json!([{"field": "tags", "op": "none_of", "value": [studio]}]),
    )
    .await;
    // Should exclude images with studio tag but still have some results
    assert!(count > 0);
    assert!(count < 55);
}

// ─── POST /images/search — combined filters ───

#[tokio::test]
async fn test_search_collection_and_model() {
    let base = spawn_app().await;
    let client = Client::new();
    let kai = get_model_uuid(&client, &base, "kai", "raw-collective").await;

    let count = search_count(
        &client,
        &base,
        json!([
            {"field": "collection", "op": "eq", "value": "raw-collective"},
            {"field": "models", "op": "any_of", "value": [kai]}
        ]),
    )
    .await;
    // kai: 3 images in raw-collective
    assert_eq!(count, 3);
}

#[tokio::test]
async fn test_search_multiple_tag_clauses() {
    let base = spawn_app().await;
    let client = Client::new();
    let natural = get_tag_uuid(&client, &base, "natural-light").await;
    let golden = get_tag_uuid(&client, &base, "golden-hour").await;
    let outdoor = get_tag_uuid(&client, &base, "outdoor").await;

    // Any of [natural-light, golden-hour] AND all of [outdoor]
    let count = search_count(
        &client,
        &base,
        json!([
            {"field": "tags", "op": "any_of", "value": [natural, golden]},
            {"field": "tags", "op": "all_of", "value": [outdoor]}
        ]),
    )
    .await;
    assert!(count > 0);
}

#[tokio::test]
async fn test_search_model_exact_and_tags() {
    let base = spawn_app().await;
    let client = Client::new();
    let raven = get_model_uuid(&client, &base, "raven", "noir-atelier").await;
    let moody = get_tag_uuid(&client, &base, "moody").await;

    let count = search_count(
        &client,
        &base,
        json!([
            {"field": "models", "op": "exact", "value": [raven]},
            {"field": "tags", "op": "any_of", "value": [moody]}
        ]),
    )
    .await;
    assert!(count > 0);
}

#[tokio::test]
async fn test_search_no_results() {
    let base = spawn_app().await;
    let client = Client::new();
    let emma = get_model_uuid(&client, &base, "emma", "lumiere-studio").await;

    // emma is in lumiere-studio, not noir-atelier
    let count = search_count(
        &client,
        &base,
        json!([
            {"field": "collection", "op": "eq", "value": "noir-atelier"},
            {"field": "models", "op": "any_of", "value": [emma]}
        ]),
    )
    .await;
    assert_eq!(count, 0);
}

// ─── POST /images/search — validation ───

#[tokio::test]
async fn test_search_invalid_op_for_field() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp = client
        .post(&format!("{base}/images/search"))
        .json(&json!({"filters": [{"field": "collection", "op": "any_of", "value": ["x"]}]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_search_exact_on_tags() {
    let base = spawn_app().await;
    let client = Client::new();
    // exact on tags is now supported — should return 200 (even with unknown UUIDs, just 0 results)
    let resp = client
        .post(&format!("{base}/images/search"))
        .json(&json!({"filters": [{"field": "tags", "op": "exact", "value": ["x"]}]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_search_malformed_json() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp = client
        .post(&format!("{base}/images/search"))
        .header("content-type", "application/json")
        .body("{bad json")
        .send()
        .await
        .unwrap();
    assert!(resp.status() == 400 || resp.status() == 422);
}

// ─── POST /images/search — response shape ───

#[tokio::test]
async fn test_search_returns_bare_image_rows() {
    let base = spawn_app().await;
    let client = Client::new();
    let results = search(
        &client,
        &base,
        json!([{"field": "gallery", "op": "eq", "value": "summer-editorial"}]),
    )
    .await;
    let arr = results.as_array().unwrap();
    assert_eq!(arr.len(), 5);
    for img in arr {
        assert!(img["uuid"].is_string());
        assert!(img["path"].is_string());
        assert!(img["collection"].is_string());
        assert!(img["gallery"].is_string());
        // No models/tags enrichment
        assert!(img.get("models").is_none());
        assert!(img.get("tags").is_none());
    }
}

// ─── POST /images/search/options ───

#[tokio::test]
async fn test_options_no_filters() {
    let base = spawn_app().await;
    let client = Client::new();
    let opts = search_options(&client, &base, json!([])).await;

    assert_eq!(opts["image_count"].as_u64().unwrap(), 55);
    assert_eq!(opts["collections"].as_array().unwrap().len(), 4);
    assert_eq!(opts["galleries"].as_array().unwrap().len(), 12);
    assert_eq!(opts["models"].as_array().unwrap().len(), 25);
    assert_eq!(opts["tags"].as_array().unwrap().len(), 34);
}

#[tokio::test]
async fn test_options_filtered_by_collection() {
    let base = spawn_app().await;
    let client = Client::new();
    let opts = search_options(
        &client,
        &base,
        json!([{"field": "collection", "op": "eq", "value": "lumiere-studio"}]),
    )
    .await;

    assert_eq!(opts["image_count"].as_u64().unwrap(), 14);
    assert_eq!(opts["collections"].as_array().unwrap().len(), 1);
    assert_eq!(opts["galleries"].as_array().unwrap().len(), 3);
    // Only models from lumiere-studio
    for m in opts["models"].as_array().unwrap() {
        assert_eq!(m["collection"].as_str().unwrap(), "lumiere-studio");
    }
}

#[tokio::test]
async fn test_options_filtered_by_model() {
    let base = spawn_app().await;
    let client = Client::new();
    let emma = get_model_uuid(&client, &base, "emma", "lumiere-studio").await;

    let opts = search_options(
        &client,
        &base,
        json!([{"field": "models", "op": "any_of", "value": [emma]}]),
    )
    .await;

    // emma has 3 images (2 solo + 1 duo with sofia)
    assert_eq!(opts["image_count"].as_u64().unwrap(), 3);
    assert_eq!(opts["collections"].as_array().unwrap().len(), 1);
}

// ─── GET /images/{uuid}/file ───

#[tokio::test]
async fn test_get_image_file_valid() {
    let base = spawn_app().await;
    let client = Client::new();

    // Get a UUID from search
    let results = search(&client, &base, json!([])).await;
    let uuid = results.as_array().unwrap()[0]["uuid"].as_str().unwrap();

    let resp = client
        .get(&format!("{base}/images/{uuid}/file"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()["content-type"].to_str().unwrap(),
        "image/jpeg"
    );
}

#[tokio::test]
async fn test_get_image_file_not_found() {
    let base = spawn_app().await;
    let client = Client::new();
    let resp = client
        .get(&format!(
            "{base}/images/00000000-0000-0000-0000-000000000000/file"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_get_image_file_content_is_jpeg() {
    let base = spawn_app().await;
    let client = Client::new();

    let results = search(&client, &base, json!([])).await;
    let uuid = results.as_array().unwrap()[0]["uuid"].as_str().unwrap();

    let body = client
        .get(&format!("{base}/images/{uuid}/file"))
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();

    // JPEG magic bytes: FF D8 FF
    assert!(body.len() > 3);
    assert_eq!(body[0], 0xFF);
    assert_eq!(body[1], 0xD8);
    assert_eq!(body[2], 0xFF);
}

// ─── PUT /images/{uuid}/tags ───

#[tokio::test]
async fn test_update_tags_set_and_verify() {
    let base = spawn_app().await;
    let client = Client::new();

    // Get an image UUID and some tag UUIDs
    let results = search(&client, &base, json!([])).await;
    let image_uuid = results.as_array().unwrap()[0]["uuid"].as_str().unwrap();
    let outdoor = get_tag_uuid(&client, &base, "outdoor").await;
    let moody = get_tag_uuid(&client, &base, "moody").await;

    // Set tags to exactly [outdoor, moody]
    let resp = client
        .put(&format!("{base}/images/{image_uuid}/tags"))
        .json(&json!({ "tag_uuids": [outdoor, moody] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify via search that the image now has exactly these tags
    let results = search(&client, &base, json!([])).await;
    let img = results
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["uuid"].as_str().unwrap() == image_uuid)
        .unwrap();
    let tag_names: Vec<&str> = img["tags"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(tag_names.len(), 2);
    assert!(tag_names.contains(&"outdoor"));
    assert!(tag_names.contains(&"moody"));
}

#[tokio::test]
async fn test_update_tags_clear_all() {
    let base = spawn_app().await;
    let client = Client::new();

    let results = search(&client, &base, json!([])).await;
    let image_uuid = results.as_array().unwrap()[0]["uuid"].as_str().unwrap();

    // Clear all tags
    let resp = client
        .put(&format!("{base}/images/{image_uuid}/tags"))
        .json(&json!({ "tag_uuids": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // Verify empty
    let results = search(&client, &base, json!([])).await;
    let img = results
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["uuid"].as_str().unwrap() == image_uuid)
        .unwrap();
    assert_eq!(img["tags"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_update_tags_invalid_image_uuid() {
    let base = spawn_app().await;
    let client = Client::new();

    let resp = client
        .put(&format!(
            "{base}/images/00000000-0000-0000-0000-000000000000/tags"
        ))
        .json(&json!({ "tag_uuids": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_update_tags_invalid_tag_uuid() {
    let base = spawn_app().await;
    let client = Client::new();

    let results = search(&client, &base, json!([])).await;
    let image_uuid = results.as_array().unwrap()[0]["uuid"].as_str().unwrap();

    let resp = client
        .put(&format!("{base}/images/{image_uuid}/tags"))
        .json(&json!({ "tag_uuids": ["nonexistent-tag-uuid"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}
