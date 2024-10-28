use std::rc::Rc;

use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;

use crate::sql;

pub async fn get_male_relatives(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<SharedString>>, Box<dyn std::error::Error>> {
    let names = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_males()).fetch_all(pool).await?;
    names.push(SharedString::from("<None>"));
    for row in rows {
        let name: String = row.get("full_name");
        names.push(slint::format!("{name}"));
    }

    Ok(names)
}

pub async fn get_mothers(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<SharedString>>, Box<dyn std::error::Error>> {
    let names = Rc::new(VecModel::default());
    let rows = sqlx::query(
        "
        SELECT 
            full_name
        FROM 
            relative
        WHERE
            LOWER(sex) = LOWER('female')
            AND age > 13
        ORDER BY
            pinned
        DESC

        ",
    )
    .fetch_all(pool)
    .await?;
    names.push(SharedString::from("<None>"));
    for row in rows {
        let name: String = row.get("full_name");
        names.push(slint::format!("{name}"));
    }
    Ok(names)
}

pub async fn get_all_employees(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let relatives: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> =
        Rc::new(VecModel::default());

    let rows = sqlx::query(&sql::get_all_employees())
        .fetch_all(pool)
        .await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
        let name: String = row.get("full_name");
        let age: i32 = row.get("age");
        let sameness: f32 = row.try_get("sameness").unwrap_or(0.0.into());
        let mother_id: i32 = row.try_get("mother_id").unwrap_or(0);
        let father_id: i32 = row.try_get("father_id").unwrap_or(0);
        let phone: String = row.try_get("phone").unwrap_or("null".into());
        let email: String = row.try_get("email").unwrap_or("null".into());
        let lost_reason: String = row.try_get("lost_reason").unwrap_or("null".into());
        let pinned: bool = row.try_get("pinned").unwrap_or(false);
        let create_at: String = row.try_get("created_at").unwrap_or("null".into());
        let updated_at: String = row.try_get("updated_at").unwrap_or("null".into());
        let employable: i32 = row.try_get("employable").unwrap_or(0);

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
        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{sameness}").into());
        items.push(slint::format!("{mother}").into());
        items.push(slint::format!("{father}").into());
        items.push(slint::format!("{phone}").into());
        items.push(slint::format!("{email}").into());
        if pinned {
            items.push(slint::format!("√").into());
        } else {
            items.push(slint::format!("").into());
        }
        items.push(slint::format!("{lost_reason}").into());
        items.push(slint::format!("{employable}").into());
        items.push(slint::format!("{create_at}").into());
        items.push(slint::format!("{updated_at}").into());
        relatives.push(items.into());
    }
    Ok(relatives)
}

pub async fn get_all_relative(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let relatives: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> =
        Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_all_relatives())
        .fetch_all(pool)
        .await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
        let name: String = row.get("full_name");
        let age: i32 = row.get("age");
        let sameness: f32 = row.try_get("sameness").unwrap_or(0.0.into());
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
        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{sameness}").into());
        items.push(slint::format!("{mother}").into());
        items.push(slint::format!("{father}").into());
        items.push(slint::format!("{phone}").into());
        items.push(slint::format!("{email}").into());
        if pinned {
            items.push(slint::format!("√").into());
        } else {
            items.push(slint::format!("").into());
        }
        items.push(slint::format!("{lost_reason}").into());
        items.push(slint::format!("{create_at}").into());
        items.push(slint::format!("{updated_at}").into());
        relatives.push(items.into());
    }
    Ok(relatives)
}

pub async fn get_female_relatives(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let females: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_female_relatives())
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
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
        let swarthy: i32 = row.try_get("swarthy").unwrap_or(0);
        let hotness: i32 = row.try_get("hotness").unwrap_or(0);
        let crazy: i32 = row.try_get("crazy").unwrap_or(0);

        let mquery = "SELECT full_name FROM relative WHERE id=$1";
        let fquery = "SELECT full_name FROM relative WHERE id=$1";

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
        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{sameness}").into());
        items.push(slint::format!("{mother}").into());
        items.push(slint::format!("{father}").into());
        items.push(slint::format!("{phone}").into());
        items.push(slint::format!("{email}").into());
        if pinned {
            items.push(slint::format!("√").into());
        } else {
            items.push(slint::format!("").into());
        }
        items.push(slint::format!("{hotness}").into());
        items.push(slint::format!("{swarthy}").into());
        items.push(slint::format!("{crazy}").into());
        items.push(slint::format!("{lost_reason}").into());
        items.push(slint::format!("{create_at}").into());
        items.push(slint::format!("{updated_at}").into());
        females.push(items.into());
    }
    Ok(females)
}

/*


async fn get_female_relatives(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let females: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_female_relatives())
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
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
        let swarthy: f32 = row.try_get("swarthy").unwrap_or(0.0);
        let hotness: f32 = row.try_get("hotness").unwrap_or(0.0);
        let crazy: f32 = row.try_get("crazy").unwrap_or(0.0);

        let mquery = "SELECT full_name FROM relative WHERE id=$1";
        let fquery = "SELECT full_name FROM relative WHERE id=$1";

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
        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{sameness}").into());
        items.push(slint::format!("{mother}").into());
        items.push(slint::format!("{father}").into());
        items.push(slint::format!("{phone}").into());
        items.push(slint::format!("{email}").into());
        items.push(slint::format!("{pinned}").into());
        items.push(slint::format!("{hotness}").into());
        items.push(slint::format!("{swarthy}").into());
        items.push(slint::format!("{crazy}").into());
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
    let rows = sqlx::query(&sql::get_all_relatives())
        .fetch_all(pool)
        .await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
        let name: String = row.get("full_name");
        let age: i32 = row.get("age");
        let sameness: f32 = row.try_get("sameness").unwrap_or(0.0.into());
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
        items.push(slint::format!("{id}").into());
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

async fn get_all_employees(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let relatives: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> =
        Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_all_employees())
        .fetch_all(pool)
        .await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
        let name: String = row.get("full_name");
        let age: i32 = row.get("age");
        let sameness: f32 = row.try_get("sameness").unwrap_or(0.0.into());
        let mother_id: i32 = row.try_get("mother_id").unwrap_or(0);
        let father_id: i32 = row.try_get("father_id").unwrap_or(0);
        let phone: String = row.try_get("phone").unwrap_or("null".into());
        let email: String = row.try_get("email").unwrap_or("null".into());
        let lost_reason: String = row.try_get("lost_reason").unwrap_or("null".into());
        let pinned: bool = row.try_get("pinned").unwrap_or(false);
        let create_at: String = row.try_get("created_at").unwrap_or("null".into());
        let updated_at: String = row.try_get("updated_at").unwrap_or("null".into());
        let employable: f32 = row.try_get("employable").unwrap_or(0.0);

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
        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{sameness}").into());
        items.push(slint::format!("{mother}").into());
        items.push(slint::format!("{father}").into());
        items.push(slint::format!("{phone}").into());
        items.push(slint::format!("{email}").into());
        items.push(slint::format!("{pinned}").into());
        items.push(slint::format!("{lost_reason}").into());
        items.push(slint::format!("{employable}").into());
        items.push(slint::format!("{create_at}").into());
        items.push(slint::format!("{updated_at}").into());
        relatives.push(items.into());
    }
    Ok(relatives)
}

*/

pub async fn get_notes_rows_for_relative(
    id: String,
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let notes: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_notes_for_relative())
        .bind(id)
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.try_get("id").unwrap_or(0);
        let text: String = row.try_get("text").unwrap_or("null".into());
        let pinned: bool = row.try_get("pinned").unwrap_or(false);
        //let create_at: String = row.try_get("created_at").unwrap_or("null".into());
        let updated_at: String = row.try_get("updated_at").unwrap_or("null".into());
        println!("{text}, {id}");

        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{text}").into());
        if pinned {
            items.push(slint::format!("√").into());
        } else {
            items.push(slint::format!("").into());
        }
        items.push(slint::format!("{updated_at}").into());
        notes.push(items.into());
    }

    Ok(notes)
}

pub async fn get_files_rows_for_relative(
    id: String,
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let notes: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_files_for_relative())
        .bind(id)
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.try_get("id").unwrap_or(0);
        let filename: String = row.try_get("filename").unwrap_or("null".into());
        let file_type: String = row.try_get("type").unwrap_or("null".into());
        let pinned: bool = row.try_get("pinned").unwrap_or(false);
        let create_at: String = row.try_get("filename_timestamp").unwrap_or("null".into());
        println!("file name: {filename}, {id}");

        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{filename}").into());
        items.push(slint::format!("{file_type}").into());
        if pinned {
            items.push(slint::format!("√").into());
        } else {
            items.push(slint::format!("").into());
        }
        items.push(slint::format!("{create_at}").into());
        notes.push(items.into());
    }

    Ok(notes)
}

pub async fn get_image_rows_for_relative(
    id: String,
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let images: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    println!("fet for id {id}");
    let rows = sqlx::query(&sql::get_images_for_relative())
        .bind(id)
        .fetch_all(pool)
        .await?;

    for row in rows {
        let items = Rc::new(VecModel::default());
        let filename: String = row.try_get("filename").unwrap_or("null".into());
        items.push(slint::format!("{filename}").into());
        images.push(items.into());
    }

    Ok(images)
}
