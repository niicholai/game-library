CREATE TABLE IF NOT EXISTS games (
                                     id TEXT PRIMARY KEY,
                                     igdb_id INTEGER,
                                     name TEXT NOT NULL,
                                     summary TEXT,
                                     storyline TEXT,
                                     rating REAL,
                                     release_date DATETIME,
                                     cover_url TEXT,
                                     screenshots TEXT, -- JSON array as string
                                     genres TEXT, -- JSON array as string
                                     platforms TEXT, -- JSON array as string
                                     developer TEXT,
                                     publisher TEXT,
                                     file_path TEXT,
                                     file_size INTEGER,
                                     is_installed BOOLEAN NOT NULL DEFAULT FALSE,
                                     created_at DATETIME NOT NULL,
                                     updated_at DATETIME NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_games_name ON games(name);
CREATE INDEX IF NOT EXISTS idx_games_igdb_id ON games(igdb_id);
CREATE INDEX IF NOT EXISTS idx_games_created_at ON games(created_at);
