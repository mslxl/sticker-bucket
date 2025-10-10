INSERT INTO migration (version) VALUES (1);

CREATE TABLE collections (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NULL,
    author TEXT NULL,
    create_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX collections_name_idx ON collections(name);
CREATE INDEX collections_create_at_idx ON collections(create_at);
CREATE INDEX collections_modified_at_idx ON collections(modified_at);
INSERT INTO collections (id, name) VALUES ('internal.inbox', 'Inbox');


CREATE TABLE asset(
    id TEXT NOT NULL PRIMARY KEY,
    typ TEXT NOT NULL CHECK (
        typ IN ('text', 'image', 'video')
    ),
    ref TEXT NOT NULL UNIQUE,
    create_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX asset_create_at_idx ON asset(create_at);

CREATE TABLE stickers (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NULL,
    description TEXT NULL,
    create_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    collection_id TEXT NOT NULL,
    FOREIGN KEY (collection_id) REFERENCES collections(id)
);
CREATE INDEX stickers_create_at_idx ON stickers(create_at);
CREATE INDEX stickers_modified_at_idx ON stickers(modified_at);
CREATE INDEX stickers_name_idx ON stickers(name);

CREATE TABLE asset_stickers (
    asset_id TEXT NOT NULL,
    sticker_id TEXT NOT NULL,
    PRIMARY KEY (asset_id, sticker_id),
    FOREIGN KEY (asset_id) REFERENCES asset(id),
    FOREIGN KEY (sticker_id) REFERENCES stickers(id)
);


CREATE TABLE tag(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL
);

CREATE INDEX tag_name_idx ON tag(name);

CREATE TABLE tag_asset(
    tag_id INTEGER NOT NULL,    
    asset_id TEXT NOT NULL,
    PRIMARY KEY (tag_id, asset_id),
    FOREIGN KEY (tag_id) REFERENCES tag(id),
    FOREIGN KEY (asset_id) REFERENCES asset(id)
);

CREATE TABLE tag_stickers(
    tag_id INTEGER NOT NULL,
    sticker_id TEXT NOT NULL,
    PRIMARY KEY (tag_id, sticker_id),
    FOREIGN KEY (tag_id) REFERENCES tag(id),
    FOREIGN KEY (sticker_id) REFERENCES stickers(id)
);

CREATE TABLE tag_collections(
    tag_id INTEGER NOT NULL,
    collection_id TEXT NOT NULL,
    PRIMARY KEY (tag_id, collection_id),
    FOREIGN KEY (tag_id) REFERENCES tag(id),
    FOREIGN KEY (collection_id) REFERENCES collections(id)
);
