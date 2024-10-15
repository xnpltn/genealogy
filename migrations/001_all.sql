-- create table for relative
CREATE TABLE relative (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME DEFAULT CURRENT_TIMESTAMP,
    sameness        FLOAT,
    lost_reason     TEXT,
    atnum           INTEGER,
    sex             TEXT NOT NULL,
    birthday        DATETIME NOT NULL,  -- Corrected from "bithday"
    age             INTEGER,
    fname           TEXT NOT NULL,
    mname           TEXT,
    lname           TEXT NOT NULL, 
    full_name       TEXT ,
    phone           TEXT UNIQUE,
    email           TEXT UNIQUE,
    mother_id       INTEGER,
    father_id       INTEGER,
    end_of_line     BOOLEAN DEFAULT 1,
    pinned          BOOLEAN DEFAULT 0,
    hotness         INTEGER DEFAULT 0,
    crazy           INTEGER DEFAULT 0,
    swarthy         INTEGER DEFAULT 0,
    employable      INTEGER DEFAULT 0,
    FOREIGN KEY (mother_id) REFERENCES relative(id),
    FOREIGN KEY (father_id) REFERENCES relative(id)
);

-- Trigger to update the updated_at field on update on relative
CREATE TRIGGER IF NOT EXISTS update_relative_updated_at
AFTER UPDATE ON relative
FOR EACH ROW
BEGIN
    UPDATE relative SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;


-- Trigger to set full_name to fname + lname on INSERT or UPDATE
CREATE TRIGGER IF NOT EXISTS set_full_name_insert
AFTER INSERT ON relative
FOR EACH ROW
BEGIN
    UPDATE relative
    SET full_name = NEW.fname || ' ' || NEW.lname
    WHERE id = NEW.id;
END;


CREATE TRIGGER IF NOT EXISTS set_full_name_update
AFTER UPDATE ON relative
FOR EACH ROW
BEGIN
    UPDATE relative
    SET full_name = NEW.fname || ' ' || NEW.lname
    WHERE id = NEW.id;
END;


-- Calculate age after creating an individual
CREATE TRIGGER calculate_age_after_insert
AFTER INSERT ON relative
BEGIN
    UPDATE relative
    SET age = CAST((julianday('now') - julianday(birthday)) / 365.25 AS INTEGER)
    WHERE id = NEW.id;
END;

-- Table file to store files related to relative
CREATE TABLE IF NOT EXISTS file (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    imported_at         DATETIME DEFAULT CURRENT_TIMESTAMP,
    imported_name       TEXT NOT NULL,
    imported_hash       TEXT NOT NULL,
    relative_id         INTEGER  NOT NULL,
    type                TEXT NOT NULL,
    size                TEXT NOT NULL,
    filename            TEXT NOT NULL,
    filename_timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP,
    filename_hashname   TEXT NOT NULL,
    file_directory      TEXT NOT NULL,
    pinned              BOOLEAN DEFAULT 0,
    FOREIGN KEY         (relative_id) REFERENCES relative(id) ON DELETE CASCADE
);

-- Trigger to update filename_timestamp on file update
CREATE TRIGGER IF NOT EXISTS update_files_updated_at
AFTER UPDATE ON file
FOR EACH ROW
BEGIN 
  UPDATE file SET filename_timestamp = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;

-- Table to store notes related to relative
CREATE TABLE IF NOT EXISTS note (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at       DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at       DATETIME DEFAULT CURRENT_TIMESTAMP,
    relative_id      INTEGER NOT NULL,
    text             TEXT NOT NULL,
    pinned           BOOLEAN DEFAULT 0,
    FOREIGN KEY      (relative_id) REFERENCES relative(id) ON DELETE CASCADE
);

-- Trigger for updating updated_at on notes
CREATE TRIGGER IF NOT EXISTS update_notes_updated_at
AFTER UPDATE ON note
FOR EACH ROW
BEGIN
    UPDATE note SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
END;
