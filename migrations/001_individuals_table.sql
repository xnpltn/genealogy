CREATE TABLE individuals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    age INTEGER NOT NULL,
    sameness TEXT,
    mother_id INTEGER,
    father_id INTEGER,
    phone TEXT,
    email TEXT,
    pinned BOOLEAN DEFAULT 0,
    lost_reason TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (mother_id) REFERENCES individuals(id),
    FOREIGN KEY (father_id) REFERENCES individuals(id)
);


-- Trigger to update the updated_at field on update
CREATE TRIGGER IF NOT EXISTS update_individuals_updated_at
AFTER UPDATE ON individuals
FOR EACH ROW
BEGIN
    UPDATE individuals SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
