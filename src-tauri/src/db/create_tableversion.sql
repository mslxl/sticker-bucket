CREATE TABLE IF NOT EXISTS table_version(
    id INTEGER NOT NULL
      CONSTRAINT table_version_pk PRIMARY KEY,
    version_code INTEGER NOT NULL
);