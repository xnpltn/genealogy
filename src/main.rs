#![allow(deprecated)]
use native_dialog::FileDialog;
use slint::*;
use slint::{StandardListViewItem, VecModel};
use sqlx::{migrate, Row};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::fs;
use std::io;
use std::rc::Rc;
use std::{env, u64};

mod sql;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut work_dir = String::from(std::format!(
        "{}/.geneapp",
        env::home_dir().unwrap().to_str().unwrap()
    ));

    if let Some(dir) = env::home_dir() {
        match fs::create_dir_all(std::format!("{}/.geneapp", dir.to_str().unwrap()).as_str()) {
            Ok(()) => println!("working dir created"),
            Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                    println!("error: {}", e);
                    fs::create_dir(".geneapp").unwrap();
                    work_dir = ".geneapp".to_string();
                } else {
                    println!("already exists");
                    work_dir = std::format!("{}/.geneapp", dir.to_str().unwrap());
                }
            }
        }
    }

    println!("{work_dir}");
    if !Sqlite::database_exists(std::format!("{}/store.db", work_dir).as_str())
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(std::format!("{}/store.db", work_dir).as_str()).await?;
        println!("db creation done");
    }

    let pool = SqlitePool::connect(std::format!("{}/store.db", work_dir).as_str()).await?;
    migrate!("./migrations").run(&pool).await?;
    let app = Main::new()?;

    let females = get_female_relatives(&pool).await?;
    let relatives = get_all_relative(&pool).await?;
    let males = get_male_relatives(&pool).await?;
    let females2 = get_mothers(&pool).await?;

    app.global::<TableData>()
        .set_females(females.clone().into());
    app.global::<TableData>()
        .set_relative(relatives.clone().into());

    app.global::<TableData>().on_current_row_changed({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |row| {
            let active_tab = weak_app.unwrap().global::<TableData>().get_active_tab();
            if active_tab == ActiveTab::Relative {
                let table_data = weak_app.unwrap().global::<TableData>().get_relative();
                let table_data = table_data.row_data(row as usize).unwrap();
                if let Some(email) = table_data.row_data(0) {
                    let email = email.text;
                    let pool = pool.clone();
                    let weak_app = weak_app.clone();

                    let _ = slint::spawn_local(async move {
                        if let Ok(relative) = get_relative_data(&email, &pool).await {
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_active_relative(relative);
                        }

                        if let Ok(rows) = sqlx::query(&sql::get_files_for_relative())
                            .bind(&email.to_string())
                            .fetch_all(&pool)
                            .await
                        {
                            let items = Rc::new(VecModel::default());
                            for row in rows {
                                let name: String = row.get("filename");
                                items.push(SharedString::from(name));
                            }
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_relative_files(items.clone().into());
                        }

                        if let Ok(row) = sqlx::query(&sql::get_note_for_relative())
                            .bind(&email.to_string())
                            .fetch_one(&pool)
                            .await
                        {
                            let note: String = row.get("text");
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_relative_note(SharedString::from(note));
                        }
                    })
                    .unwrap();
                }
            }
        }
    });

    app.global::<TableData>().set_males(males.clone().into());
    app.global::<TableData>()
        .set_females2(females2.clone().into());

    app.global::<TableData>().on_update_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();

        // the veriable 'email' is id.
        move |email, relative| {
            let pool = pool.clone();
            let weak_app = weak_app.clone();
            let mut mother_phone = String::new();
            let mut father_phone = String::new();
            let m_id = weak_app
                .unwrap()
                .global::<TableData>()
                .get_selected_mother_id();
            let f_id = weak_app
                .unwrap()
                .global::<TableData>()
                .get_selected_father_id();
            if m_id > -1 {
                mother_phone = weak_app
                    .unwrap()
                    .global::<TableData>()
                    .get_females2()
                    .row_data(m_id as usize)
                    .unwrap()
                    .row_data(2)
                    .unwrap()
                    .text
                    .to_string();
            }
            if f_id > -1 {
                father_phone = weak_app
                    .unwrap()
                    .global::<TableData>()
                    .get_males()
                    .row_data(f_id as usize)
                    .unwrap()
                    .row_data(2)
                    .unwrap()
                    .text
                    .to_string();
            }

            let _ = slint::spawn_local({
                let pool = pool.clone();
                async move {
                    let pool = pool.clone();
                    let mut mother_id_db = 0;
                    let mut father_id_db = 0;
                    if mother_phone != "" {
                        mother_id_db = sqlx::query("SELECT id FROM relative WHERE phone = $1")
                            .bind(mother_phone)
                            .fetch_one(&pool)
                            .await
                            .unwrap()
                            .get("id");
                    }

                    if father_phone != "" {
                        father_id_db = sqlx::query("SELECT id FROM relative WHERE phone = $1")
                            .bind(father_phone)
                            .fetch_one(&pool)
                            .await
                            .unwrap()
                            .get("id");
                    }

                    if mother_id_db > 0 && father_id_db > 0 {
                        let res = sqlx::query(&sql::update_both_parents())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .bind(father_id_db)
                            .bind(mother_id_db)
                            .bind(email.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                println!("updated successfully");
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    } else if mother_id_db > 0 && father_id_db <= 0 {
                        // add mother only
                        let res = sqlx::query(&sql::update_mother_only())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .bind(mother_id_db)
                            .bind(email.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                println!("updated mother successfully");
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    } else if father_id_db > 0 && mother_id_db <= 0 {
                        // update father only
                        let res = sqlx::query(&sql::update_father_only())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .bind(father_id_db)
                            .bind(email.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                println!("updated father successfully");
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    }
                }
            });
        }
    });

    app.global::<TableData>().on_add_files_for_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |row| {
            let active_tab = weak_app.unwrap().global::<TableData>().get_active_tab();
            if active_tab == ActiveTab::Relative {
                let table_data = weak_app.unwrap().global::<TableData>().get_relative();
                if let Some(table_data) = table_data.row_data(row as usize) {
                    if let Some(email) = table_data.row_data(0) {
                        let email = email.text;
                        println!("{email}");
                        let path = FileDialog::new()
                            .set_location("~/Desktop")
                            .add_filter("Image", &["jpg", "jpeg", "png"])
                            .add_filter("PDF File", &["pdf"])
                            // microsoft word, microsoft excel,powerpoint, etc
                            //.add_filter("Office File", &[""])
                            .add_filter("Media File", &["mp4", "mp3", "mkv", "avi"])
                            .show_open_single_file()
                            .unwrap();

                        if let Some(p) = path {
                            let p = Rc::new(p);
                            let mut size: u64 = 0;
                            let work_dir = work_dir.clone();
                            let _ = slint::spawn_local({
                                let pool = pool.clone();
                                let xx = Rc::clone(&p);
                                match fs::copy(
                                    xx.to_str().unwrap(),
                                    std::format!(
                                        "{work_dir}/{email}-{}",
                                        xx.file_name().unwrap().to_str().unwrap().to_string()
                                    ),
                                ) {
                                    Ok(n) => {
                                        size += n;
                                        println!("copied file")
                                    }
                                    Err(_) => weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_files_error(SharedString::from("can't add file")),
                                }

                                async move {
                                    let res = sqlx::query(&sql::add_file())
                                        .bind(xx.to_str().unwrap().to_string())
                                        .bind(xx.extension().unwrap().to_str().unwrap().to_string())
                                        .bind(size as i32)
                                        .bind(std::format!(
                                            "{email}-{}",
                                            xx.file_name().unwrap().to_str().unwrap().to_string()
                                        ))
                                        .execute(&pool)
                                        .await;
                                    match res {
                                        Ok(_) => println!("created"),
                                        Err(e) => println!("{e}"),
                                    }
                                }
                            });
                        }
                    }
                } else {
                    weak_app
                        .unwrap()
                        .global::<TableData>()
                        .set_files_error(SharedString::from("Select a relative to add files for!"));
                }
            }
        }
    });

    app.global::<TableData>().on_create_new_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |relative| {
            let pool = pool.clone();
            let weak_app = weak_app.clone();
            let mut mother_phone = String::new();
            let mut father_phone = String::new();
            let m_id = weak_app
                .unwrap()
                .global::<TableData>()
                .get_selected_mother_id();
            let f_id = weak_app
                .unwrap()
                .global::<TableData>()
                .get_selected_father_id();
            if m_id > -1 {
                mother_phone = weak_app
                    .unwrap()
                    .global::<TableData>()
                    .get_females2()
                    .row_data(m_id as usize)
                    .unwrap()
                    .row_data(2)
                    .unwrap()
                    .text
                    .to_string();
            }
            if f_id > -1 {
                father_phone = weak_app
                    .unwrap()
                    .global::<TableData>()
                    .get_males()
                    .row_data(f_id as usize)
                    .unwrap()
                    .row_data(2)
                    .unwrap()
                    .text
                    .to_string();
            }

            let _ = slint::spawn_local({
                let pool = pool.clone();
                async move {
                    let pool = pool.clone();
                    let mut mother_id_db = 0;
                    let mut father_id_db = 0;
                    if mother_phone != "" {
                        mother_id_db = sqlx::query("SELECT id FROM relative WHERE phone = $1")
                            .bind(mother_phone)
                            .fetch_one(&pool)
                            .await
                            .unwrap()
                            .get("id");
                    }

                    if father_phone != "" {
                        father_id_db = sqlx::query("SELECT id FROM relative WHERE phone = $1")
                            .bind(father_phone)
                            .fetch_one(&pool)
                            .await
                            .unwrap()
                            .get("id");
                    }

                    if mother_id_db > 0 && father_id_db > 0 {
                        // add new with both parents
                        let res = sqlx::query(&sql::create_new_relative_with_both_parents())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .bind(mother_id_db)
                            .bind(father_id_db)
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                println!("added successfully");
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    } else if mother_id_db > 0 && father_id_db <= 0 {
                        // add new with mother only
                        let res = sqlx::query(&sql::create_new_relative_with_mother_only())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .bind(mother_id_db)
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                println!("added mother successfully");
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    } else if father_id_db > 0 && mother_id_db <= 0 {
                        // update father only
                        let res = sqlx::query(&sql::create_new_relative_with_father_only())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .bind(father_id_db)
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                println!("added father successfully");
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    } else if mother_id_db <= 0 && father_id_db <= 0 {
                        // create relative with no parents
                        let res = sqlx::query(&sql::create_new_relative_no_parents())
                            .bind(relative.sameness.to_string())
                            .bind(relative.lost_reason.to_string())
                            .bind(relative.sex.to_string())
                            .bind(relative.birthday.to_string())
                            .bind(relative.first_name.to_string())
                            .bind(relative.middle_name.to_string())
                            .bind(relative.last_name.to_string())
                            .bind(relative.phone.to_string())
                            .bind(relative.email.to_string())
                            .bind(relative.pinned)
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                            }
                        }
                    }
                }
            });
        }
    });

    app.global::<TableData>().on_delete_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |id| {
            let pool = pool.clone();
            let weak_app = weak_app.unwrap().as_weak();
            let _ = slint::spawn_local({
                let pool = pool.clone();
                async move {
                    let res = sqlx::query("DELETE FROM relative WHERE id=$1")
                        .bind(id.to_string())
                        .execute(&pool)
                        .await;
                    match res {
                        Ok(_) => println!("deleted where id is 1"),
                        Err(e) => {
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_error(slint::format!("error deleting relative, {e}"));
                        }
                    }
                }
            });
        }
    });

    app.run()?;
    Ok(())
}

async fn get_relative_data(
    id: &str,
    pool: &SqlitePool,
) -> Result<Relative, Box<dyn std::error::Error>> {
    let row = sqlx::query(&sql::get_one_relative_data())
        .bind(id)
        .fetch_one(pool)
        .await?;
    let id: i32 = row.get("id");
    let fname: String = row.get("fname");
    let mname: String = row.get("mname");
    let lname: String = row.get("lname");
    let birthday: String = row.get("birthday");
    let email: String = row.get("email");
    let lost_reason: String = row.get("lost_reason");
    let phone: String = row.get("phone");
    let pinned: bool = row.get("pinned");
    let sameness: f32 = row.get("sameness");
    let sex: String = row.get("sex");
    let hotness: i32 = row.get("hotness");
    let crazy: i32 = row.get("crazy");
    let swarthy: i32 = row.get("swarthy");
    let employable: i32 = row.get("employable");

    let relative = Relative {
        id: slint::format!("{id}"),
        first_name: fname.into(),
        last_name: lname.into(),
        middle_name: mname.into(),
        birthday: birthday.into(),
        email: email.into(),
        lost_reason: lost_reason.into(),
        phone: phone.into(),
        pinned: pinned.into(),
        sameness: slint::format!("{}", sameness),
        sex: sex.into(),
        note: "".into(),
        hotness: slint::format!("{hotness}"),
        crazy: slint::format!("{crazy}"),
        swarthy: slint::format!("{swarthy}"),
        employable: slint::format!("{employable}"),
    };

    Ok(relative)
}

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

async fn get_male_relatives(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let females: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_males()).fetch_all(pool).await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let name: String = row.get("full_name");
        let phone: String = row.try_get("phone").unwrap_or("null".into());
        let age: i32 = row.try_get("age").unwrap_or(0);

        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{phone}").into());
        females.push(items.into());
    }
    Ok(females)
}

async fn get_mothers(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let females: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_females()).fetch_all(pool).await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let name: String = row.get("full_name");
        let phone: String = row.try_get("phone").unwrap_or("null".into());
        let age: i32 = row.try_get("age").unwrap_or(0);
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{phone}").into());
        females.push(items.into());
    }
    Ok(females)
}
