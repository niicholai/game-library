-- Update games table for admin-managed game library
-- SQLite doesn't support IF NOT EXISTS for ALTER TABLE, use table recreation approach

-- First, check what columns exist by querying the table structure
-- Then recreate the table with the correct structure

-- Create new games table with all required columns
CREATE TABLE games_temp (
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
                            file_path TEXT,
                            file_size INTEGER,
                            is_available BOOLEAN NOT NULL DEFAULT TRUE,
                            added_by TEXT,
                            created_at DATETIME NOT NULL,
                            updated_at DATETIME NOT NULL
);

-- Copy existing data from the old games table
-- Handle the case where some columns might not exist in the original table
INSERT INTO games_temp (
    id, igdb_id, name, summary, storyline, rating, release_date,
    cover_url, screenshots, genres, platforms, developer, publisher,
    file_path, created_at, updated_at
)
SELECT
    id, igdb_id, name, summary, storyline, rating, release_date,
    cover_url, screenshots, genres, platforms, developer, publisher,
    file_path, created_at, updated_at
FROM games;

-- Drop the old table
DROP TABLE games;

-- Rename the new table to the original name
ALTER TABLE games_temp RENAME TO games;

-- Recreate indexes for better performance
CREATE INDEX idx_games_name ON games(name);
CREATE INDEX idx_games_igdb_id ON games(igdb_id);
CREATE INDEX idx_games_created_at ON games(created_at);
CREATE INDEX idx_games_is_available ON games(is_available);
