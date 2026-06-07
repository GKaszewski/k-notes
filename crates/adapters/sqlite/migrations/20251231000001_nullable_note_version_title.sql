-- note_versions.title should be nullable to match notes where title is optional
CREATE TABLE note_versions_new (
    id TEXT PRIMARY KEY,
    note_id TEXT NOT NULL,
    title TEXT,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(note_id) REFERENCES notes(id) ON DELETE CASCADE
);

INSERT INTO note_versions_new SELECT id, note_id, NULLIF(title, ''), content, created_at FROM note_versions;

DROP TABLE note_versions;
ALTER TABLE note_versions_new RENAME TO note_versions;

CREATE INDEX idx_note_versions_note_id ON note_versions(note_id);
