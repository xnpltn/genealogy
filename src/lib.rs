

pub async fn create_persons(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let query = r#"
INSERT INTO relative 
(sameness, lost_reason, atnum, sex, birthday, age, fname, mname, lname, phone, email, mother_id, father_id, end_of_line, pinned) 
VALUES 
-- First relative (mother)
(0.95, 'None', 1001, 'Female', '1980-03-15', 44, 'Sarah', 'J.', 'Johnson', '555-0101', 'sarah.johnson@example.com', NULL, NULL, 1, 0),

-- Second relative (father)
(0.93, 'None', 1002, 'Male', '1978-07-22', 46, 'Robert', 'A.', 'Johnson', '555-0102', 'robert.johnson@example.com', NULL, NULL, 1, 0),

-- Third relative (child of Sarah and Robert)
(0.97, 'None', 1003, 'Female', '2005-11-30', 19, 'Emma', 'R.', 'Johnson', '555-0103', 'emma.johnson@example.com', 1, 2, 1, 0),

-- Fourth relative (child of Sarah and Robert)
(0.92, 'None', 1004, 'Male', '2008-09-05', 16, 'Daniel', 'M.', 'Johnson', '555-0104', 'daniel.johnson@example.com', 1, 2, 1, 0),

-- Fifth relative (no parents)
(0.89, 'Unknown', 1005, 'Male', '1990-12-10', 34, 'Thomas', 'L.', 'Smith', '555-0105', 'thomas.smith@example.com', NULL, NULL, 1, 0)
    "#;

    sqlx::query(query).execute(pool).await?;
    Ok(())
}
