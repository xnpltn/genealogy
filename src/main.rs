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
    let mut images_dir: String = std::format!("{work_dir}/images");
    let mut files_dir: String = std::format!("{work_dir}/files");

    match fs::create_dir(files_dir.clone()) {
        Ok(()) => {}
        Err(e) => {
            if e.kind() != io::ErrorKind::AlreadyExists {
                println!("error: {}", e);
                fs::create_dir("{work_dir}/files").unwrap();
                files_dir = std::format!("{work_dir}/files");
            }
        }
    }

    match fs::create_dir(images_dir.clone()) {
        Ok(()) => {}
        Err(e) => {
            if e.kind() != io::ErrorKind::AlreadyExists {
                println!("error: {}", e);
                fs::create_dir("{work_dir}/images").unwrap();
                images_dir = std::format!("{work_dir}/images");
            }
        }
    }

    println!("App directory at: {work_dir}");
    println!("Files directory {files_dir}");
    if !Sqlite::database_exists(std::format!("{}/store.db", work_dir).as_str())
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(std::format!("{}/store.db", work_dir).as_str()).await?;
        println!("No Database found.");
        println!("Creating new");
    } else {
        println!("Database Connected");
    }

    let pool = SqlitePool::connect(std::format!("{}/store.db", work_dir).as_str()).await?;
    migrate!("./migrations").run(&pool).await?;
    let app = Main::new()?;
    let update_window = UpdateWindow::new()?;
    let create_window = CreateWindow::new()?;

    let females = get_female_relatives(&pool).await?;
    let relatives = get_all_relative(&pool).await?;
    let employees = get_all_employees(&pool).await?;
    let males = get_male_relatives(&pool).await?;
    let females2 = get_mothers(&pool).await?;

    app.global::<TableData>()
        .set_females(females.clone().into());
    app.global::<TableData>()
        .set_relative(relatives.clone().into());

    app.global::<TableData>()
        .set_employees(employees.clone().into());
    app.global::<TableData>().on_show_update_window({
        let update_window = update_window.as_weak();
        move || {
            update_window.unwrap().show().unwrap();
        }
    });

    update_window.global::<TableData>().on_update_relative({
        let weak_app = update_window.as_weak();
        let pool = pool.clone();
        let app = app.as_weak();
        move |id, relative| {
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
                let app = app.clone();
                println!("updating....");
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
                            .bind(id.to_string())
                            .execute(&pool)
                            .await;

                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_update_success(SharedString::from("Relative Update"));

                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_update_eror(e.to_string().into());
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
                            .bind(id.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_update_success(SharedString::from("Updated"));
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_update_eror(e.to_string().into());
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
                            .bind(id.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_update_success(SharedString::from("Updated"));
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                            }
                            Err(e) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_error(e.to_string().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_update_eror(e.to_string().into());
                            }
                        }
                    }
                }
            });
        }
    });
    app.global::<TableData>().on_show_add_window({
        let create_window = create_window.as_weak();
        move || {
            create_window.unwrap().show().unwrap();
        }
    });

    create_window.global::<TableData>().on_create_new_relative({
        let weak_app = create_window.as_weak();
        let pool = pool.clone();
        let app = app.as_weak();
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
                let app = app.clone();
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
                            .bind(relative.employable.to_string())
                            .bind(relative.swarthy.to_string())
                            .bind(relative.hotness.to_string())
                            .bind(relative.crazy.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
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
                            .bind(relative.employable.to_string())
                            .bind(relative.swarthy.to_string())
                            .bind(relative.hotness.to_string())
                            .bind(relative.crazy.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
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
                            .bind(relative.employable.to_string())
                            .bind(relative.swarthy.to_string())
                            .bind(relative.hotness.to_string())
                            .bind(relative.crazy.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
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
                            .bind(relative.employable.to_string())
                            .bind(relative.swarthy.to_string())
                            .bind(relative.hotness.to_string())
                            .bind(relative.crazy.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {
                                let females = get_female_relatives(&pool).await.unwrap();
                                let relatives = get_all_relative(&pool).await.unwrap();
                                let employees = get_all_employees(&pool).await.unwrap();
                                let males = get_male_relatives(&pool).await.unwrap();
                                let females2 = get_mothers(&pool).await.unwrap();

                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females(females.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_relative(relatives.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_employees(employees.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_females2(females2.clone().into());
                                app.unwrap()
                                    .global::<TableData>()
                                    .set_males(males.clone().into());
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

    app.global::<TableData>().on_current_row_changed({
        let weak_app = app.as_weak();
        let update_window = update_window.as_weak();
        let pool = pool.clone();
        move |row| {
            let active_tab = weak_app.unwrap().global::<TableData>().get_active_tab();
            if active_tab == ActiveTab::Relative {
                let table_data = weak_app.unwrap().global::<TableData>().get_relative();
                let table_data = table_data.row_data(row as usize).unwrap();
                let update_window = update_window.clone();
                if let Some(id) = table_data.row_data(0) {
                    let id = id.text;
                    let pool = pool.clone();
                    let weak_app = weak_app.clone();
                    let update_window = update_window.clone();

                    let _ = slint::spawn_local(async move {
                        if let Ok(relative) = get_relative_data(&id, &pool).await {
                            println!(
                                "exctive parent: {}, {}",
                                relative.father_id, relative.mother_id
                            );
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_active_relative(relative.clone());
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_active_relative(relative.clone());
                        }

                        if let Ok(rows) = sqlx::query(&sql::get_files_for_relative())
                            .bind(&id.to_string())
                            .fetch_all(&pool)
                            .await
                        {
                            let items = Rc::new(VecModel::default());
                            for row in rows {
                                let name: String =
                                    row.try_get("filename").unwrap_or("".to_string());
                                items.push(SharedString::from(name));
                            }
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_relative_files(items.clone().into());
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_relative_files(items.clone().into());
                        }

                        if let Ok(rows) = sqlx::query(&sql::get_notes_for_relative())
                            .bind(&id.to_string())
                            .fetch_all(&pool)
                            .await
                        {
                            let items = Rc::new(VecModel::default());
                            for row in rows {
                                let name: String = row.try_get("text").unwrap_or("".to_string());
                                items.push(SharedString::from(name));
                            }
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_relative_notes(items.clone().into());
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_relative_notes(items.clone().into());
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
    update_window
        .global::<TableData>()
        .set_males(males.clone().into());
    update_window
        .global::<TableData>()
        .set_females2(females2.clone().into());

    app.global::<TableData>().on_add_image_for_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |id| {
            let weak_app = weak_app.unwrap().as_weak();
            if id == "" {
                weak_app
                    .unwrap()
                    .global::<TableData>()
                    .set_files_error(SharedString::from(
                        "No selected relative, click on one row.",
                    ));
            } else {
                if let Some(path) = FileDialog::new()
                    .set_location("~/Desktop")
                    .add_filter("Image", &["jpg", "jpeg", "png"])
                    .show_open_single_file()
                    .unwrap()
                {
                    let images_dir = images_dir.clone();
                    let _ = slint::spawn_local({
                        let weak_app = weak_app.unwrap().as_weak();
                        let pool = pool.clone();
                        if let Ok(_) = fs::copy(
                            path.to_str().unwrap(),
                            std::format!(
                                "{images_dir}/{id}-{}",
                                path.file_name().unwrap().to_str().unwrap().to_string()
                            ),
                        ) {
                        } else {
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_image_error(SharedString::from("Can't add image"));
                            println!("error adding photo");
                            return;
                        };
                        async move {
                            match sqlx::query(&sql::add_image_for_relative())
                                .bind(std::format!(
                                    "{images_dir}/{id}-{}",
                                    path.file_name().unwrap().to_str().unwrap().to_string()
                                ))
                                .bind(id.to_string())
                                .execute(&pool)
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("error: {}", e.to_string());
                                    weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_image_error(SharedString::from("Can't add image"));
                                }
                            }
                        }
                    });
                }
            }
        }
    });

    app.global::<TableData>().on_add_files_for_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |row, id| {
            if id == "" {
                weak_app
                    .unwrap()
                    .global::<TableData>()
                    .set_files_error(SharedString::from(
                        "No selected relative, click on one row.",
                    ));
            } else {
                let active_tab = weak_app.unwrap().global::<TableData>().get_active_tab();
                if active_tab == ActiveTab::Relative {
                    let table_data = weak_app.unwrap().global::<TableData>().get_relative();
                    if let Some(table_data) = table_data.row_data(row as usize) {
                        if let Some(id) = table_data.row_data(0) {
                            let id = id.text;
                            println!("{id}");
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

                                let files_dir = files_dir.clone();
                                let _ = slint::spawn_local({
                                    let pool = pool.clone();
                                    let xx = Rc::clone(&p);
                                    match fs::copy(
                                        xx.to_str().unwrap(),
                                        std::format!(
                                            "{files_dir}/{id}-{}",
                                            xx.file_name().unwrap().to_str().unwrap().to_string()
                                        ),
                                    ) {
                                        Ok(n) => {
                                            size += n;
                                            //let meta = fs::metadata(std::format!(
                                            //    "{work_dir}/{id}-{}",
                                            //    xx.file_name()
                                            //        .unwrap()
                                            //        .to_str()
                                            //        .unwrap()
                                            //        .to_string()
                                            //));
                                            //match meta {
                                            //    Ok(m) => {
                                            //        println!("meta data: {}", m.len());
                                            //    }
                                            //    Err(e) => {
                                            //        println!(" error getting file metadata {e}");
                                            //    }
                                            //}
                                        }
                                        Err(_) => {
                                            weak_app
                                                .unwrap()
                                                .global::<TableData>()
                                                .set_files_error(SharedString::from(
                                                    "can't add file",
                                                ));
                                            return;
                                        }
                                    }

                                    async move {
                                        println!("size is {size}");
                                        let res = sqlx::query(&sql::add_file())
                                            .bind(xx.to_str().unwrap().to_string())
                                            .bind(id.to_string())
                                            .bind(
                                                xx.extension()
                                                    .unwrap()
                                                    .to_str()
                                                    .unwrap()
                                                    .to_string(),
                                            )
                                            .bind(size as i32)
                                            .bind(std::format!(
                                                "{id}-{}",
                                                xx.file_name()
                                                    .unwrap()
                                                    .to_str()
                                                    .unwrap()
                                                    .to_string()
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
                        weak_app.unwrap().global::<TableData>().set_files_error(
                            SharedString::from("Select a relative to add files for!"),
                        );
                    }
                }
            }
        }
    });

    app.global::<TableData>().on_add_note_for_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        move |id, note| {
            println!("{id}{note}");
            if id == "" {
                weak_app
                    .unwrap()
                    .global::<TableData>()
                    .set_notes_error(SharedString::from("No id selected"));
                return;
            } else {
                let _ = slint::spawn_local({
                    let weak_app = weak_app.unwrap().as_weak();
                    let pool = pool.clone();
                    async move {
                        let res = sqlx::query(&sql::add_note_for_relative())
                            .bind(id.to_string())
                            .bind(note.to_string())
                            .execute(&pool)
                            .await;
                        match res {
                            Ok(_) => {}
                            Err(e) => {
                                weak_app.unwrap().global::<TableData>().set_notes_error(
                                    SharedString::from(slint::format!(
                                        "Can't add note, {}",
                                        e.to_string()
                                    )),
                                );
                            }
                        }
                    }
                });
            }
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
    let hotness: f32 = row.get("hotness");
    let crazy: f32 = row.get("crazy");
    let swarthy: f32 = row.get("swarthy");
    let employable: f32 = row.get("employable");
    let mother_id: i32 = row.get("mother_id");
    let father_id: i32 = row.get("father_id");

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
        hotness: hotness as f32,
        crazy: crazy as f32,
        swarthy: swarthy as f32,
        employable: employable as f32,
        mother_id,
        father_id,
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

async fn get_male_relatives(
    pool: &SqlitePool,
) -> Result<Rc<VecModel<ModelRc<StandardListViewItem>>>, Box<dyn std::error::Error>> {
    let females: Rc<VecModel<slint::ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    let rows = sqlx::query(&sql::get_males()).fetch_all(pool).await?;
    for row in rows {
        let items = Rc::new(VecModel::default());
        let id: i32 = row.get("id");
        let name: String = row.get("full_name");
        let phone: String = row.try_get("phone").unwrap_or("null".into());
        let age: i32 = row.try_get("age").unwrap_or(0);

        items.push(slint::format!("{id}").into());
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
        let id: i32 = row.get("id");
        items.push(slint::format!("{id}").into());
        items.push(slint::format!("{name}").into());
        items.push(slint::format!("{age}").into());
        items.push(slint::format!("{phone}").into());
        females.push(items.into());
    }
    Ok(females)
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
