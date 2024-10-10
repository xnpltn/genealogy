#![allow(unused)]
use slint::*;

use sqlx::{migrate, Row};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use std::rc::Rc;

use slint::{StandardListViewItem, VecModel};

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !Sqlite::database_exists("store.db").await.unwrap_or(false) {
        Sqlite::create_database("store.db").await?;
    }

    let pool = SqlitePool::connect("sqlite://store.db").await?;
    migrate!("./migrations").run(&pool).await?;

    let app = Main::new()?;

    let females = get_females(&pool).await?;
    let relatives = get_all_relative(&pool).await?;

    app.global::<TableData>()
        .set_females(females.clone().into());
    app.global::<TableData>()
        .set_relative(relatives.clone().into());

    app.global::<TableData>().on_current_row_changed(|row| {
        print!("{row}\n");
    });
    app.global::<TableData>().on_save_new_relative({
        let weak_app = app.as_weak();
        move |relative| {
            println!("{}", relative.sameness);
        }
    });

    app.run()?;
    Ok(())
}

async fn get_females(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let females: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());

    let rows = sqlx::query("select full_name, age, email, phone, lost_reason, sameness, father_id, mother_id , pinned ,created_at, updated_at from relative WHERE sex = 'Female' ORDER BY pinned DESC")
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let name: String = row.get("full_name");
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

        let mquery = "select full_name from relative where id=$1";
        let fquery = "select full_name from relative where id=$1";

        let mut mother: String = String::from("null");
        let mut father: String = String::from("null");

        if mother_id > 0 {
            let mrow = sqlx::query(mquery).bind(mother_id).fetch_one(pool).await?;
            mother = mrow.try_get("full_name").unwrap_or("".to_string());
        }

        if father_id > 0 {
            let prow = sqlx::query(fquery).bind(father_id).fetch_one(pool).await?;
            father = prow.try_get("full_name").unwrap_or("".to_string());
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
        females.push(items.into());
    }
    Ok(females)
}

async fn get_all_relative(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let relatives: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> =
        Rc::new(VecModel::default());

    let rows = sqlx::query("select full_name, age, email, phone, lost_reason, sameness, father_id, mother_id , pinned ,created_at, updated_at from relative ORDER BY pinned DESC")
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let name: String = row.get("full_name");
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

        let mquery = "select full_name from relative where id=$1";
        let fquery = "select full_name from relative where id=$1";

        let mut mother: String = String::from("null");
        let mut father: String = String::from("null");

        if mother_id > 0 {
            let mrow = sqlx::query(mquery).bind(mother_id).fetch_one(pool).await?;
            mother = mrow.try_get("full_name").unwrap_or("".to_string());
        }

        if father_id > 0 {
            let prow = sqlx::query(fquery).bind(father_id).fetch_one(pool).await?;
            father = prow.try_get("full_name").unwrap_or("".to_string());
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
        relatives.push(items.into());
    }
    Ok(relatives)
}
