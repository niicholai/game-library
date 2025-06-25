-- Update games table for admin-managed game library
ALTER TABLE games ADD COLUMN file_size INTEGER;
ALTER TABLE games ADD COLUMN is_available BOOLEAN NOT NULL DEFAULT TRUE;
ALTER TABLE games ADD COLUMN added_by TEXT;

-- Remove the user-specific is_installed column since that's now in user_games table
-- Note: SQLite doesn't support DROP COLUMN directly, so we'll work around this
-- by creating a new table and migrating data

-- Create new games table without is_installed
CREATE TABLE games_new (
                           id TEXT PRIMARY KEY,
                           igdb_id INTEGER,
                           name TEXT NOT NULL,
                           summary TEXT,
                           storyline TEXT,
                           rating REAL,
                           release_date DATETIME,
                           cover_url TEXT,
                           screenshots TEXT,
                           genres TEXT,
                           platforms TEXT,
                           developer TEXT,
                           publisher TEXT,
                           file_path TEXT, -- Now required for admin-added games
                           file_size INTEGER,
                           is_available BOOLEAN NOT NULL DEFAULT TRUE,
                           added_by TEXT,
                           created_at DATETIME NOT NULL,
                           updated_at DATETIME NOT NULL
);

-- Copy data from old table to new table
INSERT INTO games_new (
    id, igdb_id, name, summary, storyline, rating, release_date,
    cover_url, screenshots, genres, platforms, developer, publisher,
    file_path, created_at, updated_at
)
SELECT
    id, igdb_id, name, summary, storyline, rating, release_date,
    cover_url, screenshots, genres, platforms, developer, publisher,
    file_path, created_at, updated_at
FROM games;

-- Drop old table and rename new one
DROP TABLE games;
ALTER TABLE games_new RENAME TO games;

-- Recreate indexes
CREATE INDEX idx_games_name ON games(name);
CREATE INDEX idx_games_igdb_id ON games(igdb_id);
CREATE INDEX idx_games_created_at ON games(created_at);
CREATE INDEX idx_games_is_available ON games(is_available);
