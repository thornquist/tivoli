#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = []
# ///

"""Migrate data from old tivoli app (flat image_tags table) to new normalized schema."""

import sqlite3
import sys
import time
import uuid
from itertools import groupby
from operator import itemgetter
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
OLD_DB_PATH = BASE_DIR / ".." / ".." / "tivoli" / "data" / "tivoli.db"
NEW_DB_PATH = BASE_DIR / "data" / "tivoli.db"
BATCH_SIZE = 10_000


def create_new_db(db_path: Path) -> sqlite3.Connection:
    """Delete existing DB and create fresh schema."""
    db_path.parent.mkdir(parents=True, exist_ok=True)
    if db_path.exists():
        db_path.unlink()

    conn = sqlite3.connect(str(db_path))
    conn.execute("PRAGMA journal_mode = WAL")
    conn.execute("PRAGMA synchronous = NORMAL")
    conn.execute("PRAGMA cache_size = -64000")
    cur = conn.cursor()

    cur.execute("""
        CREATE TABLE images (
            uuid TEXT PRIMARY KEY,
            path TEXT NOT NULL,
            collection TEXT NOT NULL,
            gallery TEXT NOT NULL
        )
    """)
    cur.execute("CREATE INDEX idx_images_collection ON images(collection)")
    cur.execute("CREATE INDEX idx_images_gallery ON images(collection, gallery)")

    cur.execute("""
        CREATE TABLE models (
            uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            collection TEXT NOT NULL,
            UNIQUE(name, collection)
        )
    """)
    cur.execute("CREATE INDEX idx_models_collection ON models(collection)")

    cur.execute("""
        CREATE TABLE image_models (
            image_uuid TEXT NOT NULL REFERENCES images(uuid),
            model_uuid TEXT NOT NULL REFERENCES models(uuid),
            PRIMARY KEY (image_uuid, model_uuid)
        )
    """)

    cur.execute("""
        CREATE TABLE tag_groups (
            uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        )
    """)

    cur.execute("""
        CREATE TABLE tags (
            uuid TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            tag_group_uuid TEXT NOT NULL REFERENCES tag_groups(uuid)
        )
    """)
    cur.execute("CREATE INDEX idx_tags_group ON tags(tag_group_uuid)")

    cur.execute("""
        CREATE TABLE image_tags (
            image_uuid TEXT NOT NULL REFERENCES images(uuid),
            tag_uuid TEXT NOT NULL REFERENCES tags(uuid),
            PRIMARY KEY (image_uuid, tag_uuid)
        )
    """)

    conn.commit()
    return conn


def load_lookup_tables(old_conn: sqlite3.Connection, new_conn: sqlite3.Connection):
    """Pre-load models, tag groups, and tags. Insert into new DB. Return lookup dicts."""

    # Query distinct (model_name, collection) pairs
    model_rows = old_conn.execute("""
        SELECT DISTINCT m.tag_value, u.tag_value
        FROM image_tags m
        JOIN image_tags u ON m.image_path = u.image_path AND u.tag_type = 'universe'
        WHERE m.tag_type = 'model'
        ORDER BY u.tag_value, m.tag_value
    """).fetchall()

    model_uuid_map: dict[tuple[str, str], str] = {}
    for model_name, collection in model_rows:
        model_uuid_map[(model_name, collection)] = str(uuid.uuid4())

    # Query distinct tag values for exposure and feature
    exposure_values = [
        row[0]
        for row in old_conn.execute(
            "SELECT DISTINCT tag_value FROM image_tags WHERE tag_type='exposure' ORDER BY tag_value"
        ).fetchall()
    ]
    feature_values = [
        row[0]
        for row in old_conn.execute(
            "SELECT DISTINCT tag_value FROM image_tags WHERE tag_type='feature' ORDER BY tag_value"
        ).fetchall()
    ]

    # Generate UUIDs for tag groups and tags
    tag_group_uuids = {
        "exposure": str(uuid.uuid4()),
        "feature": str(uuid.uuid4()),
    }

    tag_uuid_map: dict[str, str] = {}
    tag_records: list[tuple[str, str, str]] = []

    for val in exposure_values:
        uid = str(uuid.uuid4())
        tag_uuid_map[("exposure", val)] = uid
        tag_records.append((uid, val, tag_group_uuids["exposure"]))

    for val in feature_values:
        uid = str(uuid.uuid4())
        tag_uuid_map[("feature", val)] = uid
        tag_records.append((uid, val, tag_group_uuids["feature"]))

    # Insert into new DB
    cur = new_conn.cursor()

    cur.executemany(
        "INSERT INTO tag_groups (uuid, name) VALUES (?, ?)",
        [(uid, name) for name, uid in tag_group_uuids.items()],
    )

    cur.executemany(
        "INSERT INTO tags (uuid, name, tag_group_uuid) VALUES (?, ?, ?)",
        tag_records,
    )

    cur.executemany(
        "INSERT INTO models (uuid, name, collection) VALUES (?, ?, ?)",
        [(uid, name, col) for (name, col), uid in model_uuid_map.items()],
    )

    new_conn.commit()

    print(f"  Models:     {len(model_uuid_map)}")
    print(f"  Tag groups: {len(tag_group_uuids)}")
    print(f"  Tags:       {len(tag_records)} ({len(exposure_values)} exposure + {len(feature_values)} feature)")

    return model_uuid_map, tag_uuid_map


def migrate_images(
    old_conn: sqlite3.Connection,
    new_conn: sqlite3.Connection,
    model_uuid_map: dict[tuple[str, str], str],
    tag_uuid_map: dict[tuple[str, str], str],
):
    """Stream images from old DB, pivot tags, insert into new DB in batches."""
    total_images = old_conn.execute(
        "SELECT COUNT(DISTINCT image_path) FROM image_tags"
    ).fetchone()[0]
    print(f"\nMigrating {total_images:,} images...")

    cursor = old_conn.execute(
        "SELECT image_path, tag_type, tag_value FROM image_tags ORDER BY image_path"
    )

    image_batch: list[tuple[str, str, str, str]] = []
    image_model_batch: list[tuple[str, str]] = []
    image_tag_batch: list[tuple[str, str]] = []

    count = 0
    skipped = 0
    total_model_links = 0
    total_tag_links = 0
    start_time = time.time()

    def flush():
        nonlocal image_batch, image_model_batch, image_tag_batch
        new_conn.executemany(
            "INSERT INTO images (uuid, path, collection, gallery) VALUES (?, ?, ?, ?)",
            image_batch,
        )
        new_conn.executemany(
            "INSERT INTO image_models (image_uuid, model_uuid) VALUES (?, ?)",
            image_model_batch,
        )
        if image_tag_batch:
            new_conn.executemany(
                "INSERT INTO image_tags (image_uuid, tag_uuid) VALUES (?, ?)",
                image_tag_batch,
            )
        new_conn.commit()
        image_batch = []
        image_model_batch = []
        image_tag_batch = []

    for image_path, rows in groupby(cursor, key=itemgetter(0)):
        collection = None
        gallery = None
        models: list[str] = []
        exposures: list[str] = []
        features: list[str] = []

        for _, tag_type, tag_value in rows:
            if tag_type == "universe":
                collection = tag_value
            elif tag_type == "gallery":
                gallery = tag_value
            elif tag_type == "model":
                models.append(tag_value)
            elif tag_type == "exposure":
                exposures.append(tag_value)
            elif tag_type == "feature":
                features.append(tag_value)

        if collection is None or gallery is None:
            skipped += 1
            continue

        # Strip collection prefix from gallery: "lsd/m10" -> "m10"
        slash_idx = gallery.find("/")
        gallery_stripped = gallery[slash_idx + 1 :] if slash_idx >= 0 else gallery

        image_uuid = str(uuid.uuid4())
        image_batch.append((image_uuid, image_path, collection, gallery_stripped))

        for model_name in models:
            key = (model_name, collection)
            if key in model_uuid_map:
                image_model_batch.append((image_uuid, model_uuid_map[key]))
                total_model_links += 1

        for exp_val in exposures:
            key = ("exposure", exp_val)
            if key in tag_uuid_map:
                image_tag_batch.append((image_uuid, tag_uuid_map[key]))
                total_tag_links += 1

        for feat_val in features:
            key = ("feature", feat_val)
            if key in tag_uuid_map:
                image_tag_batch.append((image_uuid, tag_uuid_map[key]))
                total_tag_links += 1

        count += 1
        if count % BATCH_SIZE == 0:
            flush()
            elapsed = time.time() - start_time
            rate = count / elapsed
            remaining = (total_images - count) / rate if rate > 0 else 0
            print(
                f"  {count:,}/{total_images:,} images "
                f"({elapsed:.1f}s elapsed, ~{remaining:.0f}s remaining)"
            )

    # Flush remaining
    flush()

    return {
        "images": count,
        "skipped": skipped,
        "image_models": total_model_links,
        "image_tags": total_tag_links,
    }


def main():
    old_db = OLD_DB_PATH.resolve()
    print(f"Old DB: {old_db}")
    print(f"New DB: {NEW_DB_PATH.resolve()}")
    print()

    if not old_db.exists():
        print(f"Error: Old database not found at {old_db}")
        sys.exit(1)

    start = time.time()

    old_conn = sqlite3.connect(f"file:{old_db}?mode=ro", uri=True)
    old_conn.execute("PRAGMA journal_mode = WAL")

    print("Creating new database...")
    new_conn = create_new_db(NEW_DB_PATH)

    print("Loading lookup tables...")
    model_uuid_map, tag_uuid_map = load_lookup_tables(old_conn, new_conn)

    stats = migrate_images(old_conn, new_conn, model_uuid_map, tag_uuid_map)

    old_conn.close()
    new_conn.close()

    elapsed = time.time() - start
    db_size_mb = NEW_DB_PATH.stat().st_size / (1024 * 1024)

    print(f"\nMigration complete! ({elapsed:.1f}s)")
    print(f"  Images:       {stats['images']:,}")
    print(f"  Image-models: {stats['image_models']:,}")
    print(f"  Image-tags:   {stats['image_tags']:,}")
    if stats["skipped"]:
        print(f"  Skipped:      {stats['skipped']:,}")
    print(f"  New DB size:  {db_size_mb:.1f} MB")


if __name__ == "__main__":
    main()
