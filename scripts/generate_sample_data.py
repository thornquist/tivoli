#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.10"
# dependencies = ["Pillow"]
# ///

"""Generate sample gallery images and a SQLite database for the Tivoli image viewer."""

import os
import sqlite3
import uuid
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

BASE_DIR = Path(__file__).resolve().parent.parent
GALLERIES_DIR = BASE_DIR / "galleries"
DB_PATH = BASE_DIR / "data" / "tivoli.db"

SIZES = {
    "landscape_large": (1920, 1280),
    "landscape_small": (1600, 1067),
    "portrait_large": (1280, 1920),
    "portrait_small": (1067, 1600),
}

# Each image tuple: (filename, size_key, [model_names])
COLLECTIONS = {
    "lumiere-studio": {
        "bg": (62, 48, 42),
        "accent": (120, 90, 70),
        "galleries": {
            "summer-editorial": [
                ("emma-white-dress.jpg", "portrait_large", ["emma"]),
                ("emma-garden-bench.jpg", "landscape_large", ["emma"]),
                ("sofia-sunhat.jpg", "portrait_small", ["sofia"]),
                ("sofia-floral-close.jpg", "portrait_large", ["sofia"]),
                ("emma-sofia-duo.jpg", "landscape_small", ["emma", "sofia"]),
            ],
            "bridal-collection": [
                ("clara-veil-portrait.jpg", "portrait_large", ["clara"]),
                ("clara-bouquet-hold.jpg", "portrait_small", ["clara"]),
                ("clara-window-light.jpg", "landscape_large", ["clara"]),
                ("lena-lace-detail.jpg", "portrait_large", ["lena"]),
                ("lena-mirror-shot.jpg", "landscape_small", ["lena"]),
            ],
            "corporate-headshots": [
                ("james-grey-suit.jpg", "portrait_large", ["james"]),
                ("anna-blazer.jpg", "portrait_small", ["anna"]),
                ("marcus-casual.jpg", "portrait_large", ["marcus"]),
                ("anna-standing.jpg", "landscape_large", ["anna"]),
            ],
        },
    },
    "raw-collective": {
        "bg": (40, 44, 52),
        "accent": (80, 88, 100),
        "galleries": {
            "street-fashion": [
                ("kai-denim-alley.jpg", "portrait_large", ["kai"]),
                ("kai-graffiti-wall.jpg", "landscape_large", ["kai"]),
                ("zara-leather-jacket.jpg", "portrait_small", ["zara"]),
                ("zara-rooftop-pose.jpg", "landscape_small", ["zara"]),
                ("kai-zara-crosswalk.jpg", "landscape_large", ["kai", "zara"]),
            ],
            "grunge-series": [
                ("milo-warehouse.jpg", "landscape_large", ["milo"]),
                ("milo-chain-link.jpg", "portrait_large", ["milo"]),
                ("nina-smoke.jpg", "portrait_small", ["nina"]),
                ("nina-fire-escape.jpg", "portrait_large", ["nina"]),
            ],
            "tattoo-portraits": [
                ("diego-arm-detail.jpg", "landscape_small", ["diego"]),
                ("diego-half-sleeve.jpg", "portrait_large", ["diego"]),
                ("suki-back-piece.jpg", "portrait_large", ["suki"]),
                ("suki-close-up.jpg", "portrait_small", ["suki"]),
                ("diego-suki-duo.jpg", "landscape_large", ["diego", "suki"]),
            ],
        },
    },
    "golden-hour-photo": {
        "bg": (110, 75, 38),
        "accent": (180, 130, 60),
        "galleries": {
            "sunset-session": [
                ("olivia-beach-glow.jpg", "landscape_large", ["olivia"]),
                ("olivia-silhouette.jpg", "portrait_large", ["olivia"]),
                ("noah-cliff-edge.jpg", "landscape_small", ["noah"]),
                ("noah-golden-profile.jpg", "portrait_small", ["noah"]),
            ],
            "wildflower-shoot": [
                ("ava-meadow-twirl.jpg", "landscape_large", ["ava"]),
                ("ava-poppy-close.jpg", "portrait_large", ["ava"]),
                ("lily-daisy-crown.jpg", "portrait_small", ["lily"]),
                ("lily-tall-grass.jpg", "landscape_small", ["lily"]),
                ("ava-lily-laughing.jpg", "landscape_large", ["ava", "lily"]),
            ],
            "beach-portraits": [
                ("ethan-surf-board.jpg", "landscape_large", ["ethan"]),
                ("ethan-wet-hair.jpg", "portrait_large", ["ethan"]),
                ("maya-sand-dunes.jpg", "portrait_small", ["maya"]),
                ("maya-wave-splash.jpg", "landscape_small", ["maya"]),
            ],
        },
    },
    "noir-atelier": {
        "bg": (25, 25, 30),
        "accent": (55, 55, 65),
        "galleries": {
            "film-noir": [
                ("vincent-fedora.jpg", "portrait_large", ["vincent"]),
                ("vincent-shadow-play.jpg", "landscape_large", ["vincent"]),
                ("iris-cigarette-holder.jpg", "portrait_small", ["iris"]),
                ("iris-venetian-blind.jpg", "landscape_small", ["iris"]),
                ("vincent-iris-standoff.jpg", "landscape_large", ["vincent", "iris"]),
            ],
            "smoke-and-shadows": [
                ("raven-haze.jpg", "portrait_large", ["raven"]),
                ("raven-backlit.jpg", "landscape_large", ["raven"]),
                ("raven-mirror-fog.jpg", "portrait_small", ["raven"]),
                ("ash-spotlight.jpg", "portrait_large", ["ash"]),
            ],
            "monochrome-series": [
                ("elena-high-contrast.jpg", "portrait_large", ["elena"]),
                ("elena-fabric-drape.jpg", "landscape_small", ["elena"]),
                ("leo-stark-profile.jpg", "portrait_small", ["leo"]),
                ("leo-hands-close.jpg", "landscape_large", ["leo"]),
                ("elena-leo-symmetry.jpg", "landscape_large", ["elena", "leo"]),
            ],
        },
    },
}


def load_fonts():
    """Try to load macOS system fonts, fall back to Pillow default."""
    font_paths = [
        "/System/Library/Fonts/Helvetica.ttc",
        "/System/Library/Fonts/SFNSText.ttf",
        "/System/Library/Fonts/Geneva.ttf",
    ]
    for font_path in font_paths:
        if os.path.exists(font_path):
            try:
                return (
                    ImageFont.truetype(font_path, 56),
                    ImageFont.truetype(font_path, 40),
                    ImageFont.truetype(font_path, 30),
                )
            except Exception:
                continue
    default = ImageFont.load_default()
    return default, default, default


def generate_image(width, height, collection, gallery, image_name, bg, accent, fonts, output_path):
    """Generate a single sample image with text overlay."""
    img = Image.new("RGB", (width, height), bg)
    draw = ImageDraw.Draw(img)

    # Draw gradient accent on top 30%
    accent_height = int(height * 0.3)
    for y in range(accent_height):
        factor = y / accent_height
        r = int(bg[0] + (accent[0] - bg[0]) * factor)
        g = int(bg[1] + (accent[1] - bg[1]) * factor)
        b = int(bg[2] + (accent[2] - bg[2]) * factor)
        draw.line([(0, y), (width, y)], fill=(r, g, b))

    # Draw a subtle border line
    border_color = tuple(min(c + 40, 255) for c in bg)
    draw.rectangle([0, 0, width - 1, height - 1], outline=border_color, width=3)

    font_large, font_medium, font_small = fonts
    display_name = image_name.replace(".jpg", "").replace("-", " ")

    # Measure text
    lines = [
        (collection.upper(), font_large),
        (gallery, font_medium),
        (display_name, font_small),
    ]
    line_bboxes = []
    for text, font in lines:
        bbox = draw.textbbox((0, 0), text, font=font)
        line_bboxes.append((bbox[2] - bbox[0], bbox[3] - bbox[1]))

    line_spacing = 16
    total_text_height = sum(h for _, h in line_bboxes) + line_spacing * (len(lines) - 1)
    max_text_width = max(w for w, _ in line_bboxes)

    # Draw semi-transparent backdrop
    pad_x, pad_y = 40, 30
    rect_w = max_text_width + pad_x * 2
    rect_h = total_text_height + pad_y * 2
    rect_x = (width - rect_w) // 2
    rect_y = (height - rect_h) // 2

    overlay = Image.new("RGBA", (width, height), (0, 0, 0, 0))
    overlay_draw = ImageDraw.Draw(overlay)
    overlay_draw.rounded_rectangle(
        [rect_x, rect_y, rect_x + rect_w, rect_y + rect_h],
        radius=12,
        fill=(0, 0, 0, 140),
    )
    img = Image.alpha_composite(img.convert("RGBA"), overlay).convert("RGB")
    draw = ImageDraw.Draw(img)

    # Draw text lines centered
    y_cursor = rect_y + pad_y
    for (text, font), (tw, th) in zip(lines, line_bboxes):
        x = (width - tw) // 2
        draw.text((x, y_cursor), text, fill=(255, 255, 255), font=font)
        y_cursor += th + line_spacing

    output_path.parent.mkdir(parents=True, exist_ok=True)
    img.save(str(output_path), "JPEG", quality=85)


def create_database(db_path, image_records, image_model_links):
    """Create SQLite database and insert image and model records."""
    db_path.parent.mkdir(parents=True, exist_ok=True)
    if db_path.exists():
        db_path.unlink()

    conn = sqlite3.connect(str(db_path))
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

    cur.executemany(
        "INSERT INTO images (uuid, path, collection, gallery) VALUES (?, ?, ?, ?)",
        image_records,
    )

    # Collect unique models per collection, assign uuids
    model_uuids = {}  # (name, collection) -> uuid
    for image_uuid, model_names, collection in image_model_links:
        for name in model_names:
            key = (name, collection)
            if key not in model_uuids:
                model_uuids[key] = str(uuid.uuid4())

    cur.executemany(
        "INSERT INTO models (uuid, name, collection) VALUES (?, ?, ?)",
        [(uid, name, col) for (name, col), uid in model_uuids.items()],
    )

    # Insert image-model links
    link_records = []
    for image_uuid, model_names, collection in image_model_links:
        for name in model_names:
            link_records.append((image_uuid, model_uuids[(name, collection)]))

    cur.executemany(
        "INSERT INTO image_models (image_uuid, model_uuid) VALUES (?, ?)",
        link_records,
    )

    conn.commit()
    conn.close()
    return len(model_uuids)


def main():
    fonts = load_fonts()
    image_records = []
    image_model_links = []  # (image_uuid, [model_names], collection)
    image_count = 0

    for collection_name, collection_data in COLLECTIONS.items():
        bg = collection_data["bg"]
        accent = collection_data["accent"]
        for gallery_name, images in collection_data["galleries"].items():
            for image_name, size_key, model_names in images:
                width, height = SIZES[size_key]
                rel_path = f"{collection_name}/{gallery_name}/{image_name}"
                output_path = GALLERIES_DIR / rel_path

                generate_image(
                    width, height, collection_name, gallery_name, image_name,
                    bg, accent, fonts, output_path,
                )

                image_uuid = str(uuid.uuid4())
                image_records.append((image_uuid, rel_path, collection_name, gallery_name))
                image_model_links.append((image_uuid, model_names, collection_name))
                image_count += 1
                print(f"  Generated: {rel_path} ({width}x{height}) [{', '.join(model_names)}]")

    model_count = create_database(DB_PATH, image_records, image_model_links)

    collections = len(COLLECTIONS)
    galleries = sum(len(c["galleries"]) for c in COLLECTIONS.values())
    print(f"\nDone! {image_count} images, {model_count} models across {collections} studios and {galleries} shoots.")
    print(f"Database: {DB_PATH}")


if __name__ == "__main__":
    main()
