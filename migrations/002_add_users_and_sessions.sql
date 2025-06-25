-- Add user management tables
CREATE TABLE users (
                       id TEXT PRIMARY KEY,
                       username TEXT UNIQUE NOT NULL,
                       password_hash TEXT NOT NULL,
                       email TEXT,
                       is_admin BOOLEAN NOT NULL DEFAULT FALSE,
                       created_at DATETIME NOT NULL,
                       updated_at DATETIME NOT NULL
);

CREATE TABLE user_games (
                            id TEXT PRIMARY KEY,
                            user_id TEXT NOT NULL,
                            game_id TEXT NOT NULL,
                            is_installed BOOLEAN NOT NULL DEFAULT FALSE,
                            install_path TEXT,
                            installed_at DATETIME,
                            last_played DATETIME,
                            play_time_minutes INTEGER DEFAULT 0,
                            created_at DATETIME NOT NULL,
                            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
                            FOREIGN KEY (game_id) REFERENCES games(id) ON DELETE CASCADE,
                            UNIQUE(user_id, game_id)
);

CREATE TABLE sessions (
                          id TEXT PRIMARY KEY,
                          user_id TEXT NOT NULL,
                          token TEXT UNIQUE NOT NULL,
                          expires_at DATETIME NOT NULL,
                          created_at DATETIME NOT NULL,
                          FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX idx_user_games_user_id ON user_games(user_id);
CREATE INDEX idx_user_games_game_id ON user_games(game_id);
CREATE INDEX idx_sessions_token ON sessions(token);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
