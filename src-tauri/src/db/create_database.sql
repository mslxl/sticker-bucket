CREATE TABLE meme(
    id         INTEGER
        CONSTRAINT meme_pk
            PRIMARY KEY AUTOINCREMENT,
    content    TEXT NOT NULL,
    extra_data TEXT,
    summary    TEXT NOT NULL,
    desc       TEXT,
    thumbnail  TEXT,
    create_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_time DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT meme_pk2
        UNIQUE (id, content, extra_data)
);

CREATE TRIGGER [UpdateUpdateTime] AFTER UPDATE ON meme FOR EACH ROW
BEGIN
    UPDATE meme SET update_time = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

CREATE TABLE tag(
    id        INTEGER NOT NULL
        CONSTRAINT tag_pk
            PRIMARY KEY,
    namespace TEXT    NOT NULL,
    value     INTEGER NOT NULL,
    CONSTRAINT tag_pk2
        UNIQUE (namespace, value)
);

CREATE TABLE meme_tag(
    tag_id  INTEGER NOT NULL
        CONSTRAINT table_name_tag_id_fk
            REFERENCES tag,
    meme_id INTEGER NOT NULL
        CONSTRAINT table_name_meme_id_fk
            REFERENCES meme,
    CONSTRAINT table_name_pk
        PRIMARY KEY (tag_id, meme_id)
);

