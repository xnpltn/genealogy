-- Table to store notes related to individuals
CREATE TABLE IF NOT EXISTS notes (
    author TEXT NOT NULL,
    text TEXT NOT NULL,
    pinned BOOLEAN DEFAULT 0,
    individual_id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (individual_id) REFERENCES Individuals(id)
);

-- Triggers for updating updated_at
CREATE TRIGGER IF NOT EXISTS update_notes_updated_at
AFTER UPDATE ON notes
FOR EACH ROW
BEGIN
    UPDATE notes SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
