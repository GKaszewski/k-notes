CREATE TABLE IF NOT EXISTS note_links (
    source_note_id TEXT NOT NULL,
    target_note_id TEXT NOT NULL,
    score REAL NOT NULL,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (source_note_id, target_note_id),
    FOREIGN KEY (source_note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_note_id) REFERENCES notes(id) ON DELETE CASCADE
);

CREATE INDEX idx_note_links_source ON note_links(source_note_id);
CREATE INDEX idx_note_links_target ON note_links(target_note_id);
