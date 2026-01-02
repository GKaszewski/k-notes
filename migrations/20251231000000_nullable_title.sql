-- Allow NULL titles in notes table
-- SQLite doesn't support ALTER COLUMN, so we need to recreate the table

-- Step 1: Create new table with nullable title
CREATE TABLE notes_new (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT,  -- Now nullable
    content TEXT NOT NULL DEFAULT '',
    color TEXT NOT NULL DEFAULT 'DEFAULT',
    is_pinned INTEGER NOT NULL DEFAULT 0,
    is_archived INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Step 2: Copy data from old table
INSERT INTO notes_new (id, user_id, title, content, color, is_pinned, is_archived, created_at, updated_at)
SELECT id, user_id, title, content, color, is_pinned, is_archived, created_at, updated_at FROM notes;

-- Step 3: Drop old table
DROP TABLE notes;

-- Step 4: Rename new table
ALTER TABLE notes_new RENAME TO notes;

-- Step 5: Recreate indexes
CREATE INDEX idx_notes_user_id ON notes(user_id);
CREATE INDEX idx_notes_is_pinned ON notes(is_pinned);
CREATE INDEX idx_notes_is_archived ON notes(is_archived);
CREATE INDEX idx_notes_updated_at ON notes(updated_at);

-- Step 6: Recreate FTS triggers
CREATE TRIGGER notes_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, content) VALUES (NEW.rowid, COALESCE(NEW.title, ''), NEW.content);
END;

CREATE TRIGGER notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES('delete', OLD.rowid, COALESCE(OLD.title, ''), OLD.content);
END;

CREATE TRIGGER notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content) VALUES('delete', OLD.rowid, COALESCE(OLD.title, ''), OLD.content);
    INSERT INTO notes_fts(rowid, title, content) VALUES (NEW.rowid, COALESCE(NEW.title, ''), NEW.content);
END;
