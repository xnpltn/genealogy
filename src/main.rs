use slint::*;

use sqlx::{migrate, Row};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use std::rc::Rc;

use slint::{StandardListViewItem, VecModel};

slint::include_modules!();

async fn _create_persons(pool: &sqlx::SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    let query = "
INSERT INTO Individuals (name, age, sameness, mother_id, father_id, phone, email, pinned, lost_reason)
VALUES 
    ('John Doe', 35, 'Group A', NULL, NULL, '555-1234', 'john.doe@email.com', 0, NULL),
    ('Jane Smith', 28, 'Group B', NULL, NULL, '555-5678', 'jane.smith@email.com', 1, NULL),
    ('Alice Johnson', 42, 'Group A', 2, NULL, '555-9876', 'alice.johnson@email.com', 0, 'Moved away'),
    ('Bob Brown', 19, 'Group C', 2, 1, '555-4321', 'bob.brown@email.com', 0, NULL),
    ('Emily Davis', 31, 'Group B', NULL, NULL, '555-8765', 'emily.davis@email.com', 1, NULL);
";
    sqlx::query(query).execute(pool).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !Sqlite::database_exists("store.db").await.unwrap_or(false) {
        Sqlite::create_database("store.db").await?;
    }

    let pool = SqlitePool::connect("sqlite://store.db").await?;
    migrate!("./migrations").run(&pool).await?;

    let app = Main::new()?;
    let row_data: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());

    let rows = sqlx::query("select name, age, email, phone, lost_reason, sameness, father_id, mother_id , pinned ,created_at, updated_at from individuals ORDER BY pinned DESC")
        .fetch_all(&pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let name: String = row.get("name");
        let age: i32 = row.get("age");
        let sameness: String = row.try_get("sameness").unwrap_or("null".into());
        let mother_id: i32 = row.try_get("mother_id").unwrap_or(0);
        let father_id: i32 = row.try_get("father_id").unwrap_or(0);
        let phone: String = row.try_get("phone").unwrap_or("null".into());
        let email: String = row.try_get("email").unwrap_or("null".into());
        let lost_reason: String = row.try_get("lost_reason").unwrap_or("null".into());
        let pinned: bool = row.try_get("pinned").unwrap_or(false);
        let create_at: String = row.try_get("created_at").unwrap_or("null".into());
        let updated_at: String = row.try_get("updated_at").unwrap_or("null".into());

        let mquery = "select name from individuals where id=$1";
        let fquery = "select name from individuals where id=$1";

        let mut mother: String = String::from("null");
        let mut father: String = String::from("null");

        if mother_id > 0 {
            let mrow = sqlx::query(mquery).bind(mother_id).fetch_one(&pool).await?;
            mother = mrow.try_get("name").unwrap_or("".to_string());
        }

        if father_id > 0 {
            let prow = sqlx::query(fquery).bind(father_id).fetch_one(&pool).await?;
            father = prow.try_get("name").unwrap_or("".to_string());
        }
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{sameness}").into());
        items.push(slint::format!("{mother}").into());
        items.push(slint::format!("{father}").into());
        items.push(slint::format!("{phone}").into());
        items.push(slint::format!("{email}").into());
        items.push(slint::format!("{pinned}").into());
        items.push(slint::format!("{lost_reason}").into());
        items.push(slint::format!("{create_at}").into());
        items.push(slint::format!("{updated_at}").into());
        row_data.push(items.into());
    }

    app.global::<TableData>()
        .set_row_data(row_data.clone().into());

    app.global::<TableData>().on_current_row_changed(|row| {
        print!("{row}\n");
    });

    app.run()?;
    Ok(())
}
