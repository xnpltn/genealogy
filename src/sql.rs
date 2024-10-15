pub fn create_new_relative_no_parents() -> String {
    let query = r#"
            INSERT INTO relative 
                (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#;
    query.to_string()
}

pub fn create_new_relative_with_mother_only() -> String {
    let query = r#"
            INSERT INTO relative 
                (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, mother_id)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#;
    query.to_string()
}

pub fn create_new_relative_with_father_only() -> String {
    let query = r#"
            INSERT INTO relative 
                (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, father_id)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#;
    query.to_string()
}

pub fn create_new_relative_with_both_parents() -> String {
    let query = r#"
            INSERT INTO relative 
                (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, mother_id, father_id)
            VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#;
    query.to_string()
}

pub fn get_all_relatives() -> String {
    let query = r#"
        SELECT 
            id,
            full_name,
            age,
            email,
            phone,
            lost_reason,
            sameness,
            father_id,
            mother_id,
            pinned,
            created_at,
            updated_at
        FROM 
            relative
        ORDER BY
            pinned
        DESC
        ;
    "#;

    query.to_string()
}

pub fn get_female_relatives() -> String {
    let query = r#"
        SELECT 
            id,
            full_name,
            age,
            email,
            phone,
            lost_reason,
            sameness,
            father_id,
            mother_id,
            pinned,
            created_at,
            updated_at
        FROM 
            relative
        WHERE
            LOWER(sex) = LOWER('female')
        ORDER BY
            pinned
        DESC
        ;
    "#;

    query.to_string()
}

pub fn get_one_relative_data() -> String {
    let query = r#"
        SELECT
            *
        FROM
            relative
        WHERE 
            id = $1
        ;
    "#;
    query.to_string()
}
pub fn update_mother_only() -> String {
    let query = r#"
            UPDATE relative 
            SET 
                sameness = $1,
                lost_reason = $2,
                sex = $3,
                birthday = $4,
                fname = $5,
                mname = $6,
                lname = $7,
                phone = $8,
                email = $9,
                pinned = $10,
                mother_id = $11
            WHERE 
                id = $12
            ;

    "#
    .to_string();
    query
}

pub fn update_father_only() -> String {
    let query = r#"
            UPDATE relative 
            SET 
                sameness = $1,
                lost_reason = $2,
                sex = $3,
                birthday = $4,
                fname = $5,
                mname = $6,
                lname = $7,
                phone = $8,
                email = $9,
                pinned = $10,
                father_id = $11
            WHERE 
                id = $12
            ;
    "#
    .to_string();
    query
}

pub fn update_both_parents() -> String {
    let query = r#"
            UPDATE relative 
            SET 
                sameness = $1,
                lost_reason = $2,
                sex = $3,
                birthday = $4,
                fname = $5,
                mname = $6,
                lname = $7,
                phone = $8,
                email = $9,
                pinned = $10,
                father_id = $11,
                mother_id = $12
            WHERE 
                id = $13
            ;
    "#
    .to_string();
    query
}

pub fn get_females() -> String {
    let query = r#"
        SELECT 
            id,
            full_name,
            phone,
            age
        FROM 
            relative
        WHERE
            LOWER(sex) = LOWER('female')
        ORDER BY
            pinned
        DESC
        ;
    "#;
    query.to_string()
}

pub fn get_males() -> String {
    let query = r#"
        SELECT 
            id,
            full_name,
            age,
            phone
        FROM 
            relative
        WHERE
            LOWER(sex) = LOWER('male')
        ORDER BY
            pinned
        DESC
        ;
    "#;
    query.to_string()
}

/*
*

CREATE TABLE IF NOT EXISTS file (
    imported_at         DATETIME DEFAULT CURRENT_TIMESTAMP,
    imported_name       TEXT NOT NULL,
    imported_hash       TEXT NOT NULL,
    relative_id      INTEGER PRIMARY KEY NOT NULL,
    type                TEXT NOT NULL,
    size                TEXT NOT NULL,
    filename            TEXT NOT NULL,
    filename_timestamp  DATETIME DEFAULT CURRENT_TIMESTAMP,
    filename_hashname   TEXT NOT NULL,
    file_directory      TEXT NOT NULL,
    pinned              BOOLEAN DEFAULT 0,
    FOREIGN KEY         (relative_id) REFERENCES relative(id)
);
* */
pub fn add_file() -> String {
    let query = r#"
               INSERT INTO file (
                   imported_name,
                   imported_hash,
                   relative_id,
                   type,
                   size,
                   filename,
                   filename_hashname,
                   file_directory
               ) VALUES (
                    $1,
                    'xzy',
                    1,
                    $2,
                    $3,
                    $4,
                    'xzy',
                    'file'
               )
               
    "#;

    query.to_string()
}

pub fn get_files_for_relative() -> String {
    let query = r#"
        SELECT * FROM file WHERE relative_id = (
            SELECT id FROM relative WHERE id = $1
        );
    "#;
    query.to_string()
}

pub fn get_note_for_relative() -> String {
    let query = r#"
        SELECT * FROM note WHERE relative_id = (
            SELECT id FROM relative WHERE id = $1
        );
    "#;
    query.to_string()
}
