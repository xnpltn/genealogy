pub fn create_new_relative_no_parents() -> String {
    let query = r#"
        INSERT INTO relative 
            (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, employable, swarthy, hotness, crazy)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#;
    query.to_string()
}

pub fn create_new_relative_with_mother_only() -> String {
    let query = r#"
        INSERT INTO relative 
            (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, mother_id, employable, swarthy, hotness, crazy)
    VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        "#;
    query.to_string()
}

pub fn create_new_relative_with_father_only() -> String {
    let query = r#"
        INSERT INTO relative 
            (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, father_id, employable, swarthy, hotness, crazy)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
"#;
    query.to_string()
}

pub fn create_new_relative_with_both_parents() -> String {
    let query = r#"
        INSERT INTO relative 
            (sameness, lost_reason, sex, birthday, fname, mname, lname, phone, email, pinned, mother_id, father_id, employable, swarthy, hotness, crazy)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
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
            *
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

pub fn get_all_employees() -> String {
    let query = r#"
        SELECT 
            *
        FROM 
            relative 
        WHERE 
            employable > 0
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

pub fn add_file() -> String {
    let query = r#"
               INSERT INTO file 
                  (imported_name, imported_hash, relative_id, type, size, filename, filename_hashname, file_directory
               ) 
               VALUES 
                  ( $1,'xzy', $2 , $3, $4, $5,'xzy','files')
            ;
               
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

pub fn get_notes_for_relative() -> String {
    let query = r#"
        SELECT * FROM note WHERE relative_id = (
            SELECT id FROM relative WHERE id = $1
        );
    "#;
    query.to_string()
}

pub fn add_image_for_relative() -> String {
    let query = r#"
        INSERT INTO image
            (filename, relative_id)
        VALUES ($1, $2)
        ;
    "#;
    query.to_string()
}

pub fn add_note_for_relative() -> String {
    let query = r#"
        INSERT INTO note
            (relative_id, text)
        VALUES
            ($1, $1)
    ;"#;
    query.to_string()
}
