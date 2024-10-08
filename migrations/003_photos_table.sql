-- Table to store images related to individuals
CREATE TABLE IF NOT EXISTS images (
    location TEXT NOT NULL,
    alt TEXT,
    individual_id INTEGER PRIMARY KEY NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (individual_id) REFERENCES individuals(id)
);


-- Trigger to update the updated_at field on update
CREATE TRIGGER IF NOT EXISTS update_images_updated_at
AFTER UPDATE ON images
FOR EACH ROW
BEGIN
    UPDATE images SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
