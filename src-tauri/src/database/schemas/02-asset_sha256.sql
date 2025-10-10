INSERT INTO migration (version) VALUES (2);

ALTER TABLE asset ADD COLUMN sha256 TEXT NULL;

CREATE INDEX asset_sha256_idx ON asset(sha256);
