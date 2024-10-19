#![allow(deprecated)]
use native_dialog::FileDialog;
use slint::*;
use sqlx::{migrate, Row};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::fs;
use std::io;
use std::rc::Rc;
use std::{env, u64};

mod repo;
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

    let females = repo::get_female_relatives(&pool).await?;
    let relatives = repo::get_all_relative(&pool).await?;
    let employees = repo::get_all_employees(&pool).await?;
    let males = repo::get_male_relatives(&pool).await?;
    let females2 = repo::get_mothers(&pool).await?;

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

    app.global::<TableData>().on_show_add_window({
        let update_weak_window = update_window.as_weak();
        let pool = pool.clone();
        let app = app.as_weak();
        move || {
            let _ = slint::spawn_local({
                let pool = pool.clone();
                let update_weak_window = update_weak_window.clone();
                let app = app.clone();
                async move {
                    let females = repo::get_female_relatives(&pool).await.unwrap();
                    let males = repo::get_male_relatives(&pool).await.unwrap();
                    let females2 = repo::get_mothers(&pool).await.unwrap();
                    let create_window = CreateWindow::new().unwrap();
                    create_window
                        .global::<TableData>()
                        .set_females(females.clone().into());
                    create_window
                        .global::<TableData>()
                        .set_males(males.clone().into());
                    create_window
                        .global::<TableData>()
                        .set_females2(females2.clone().into());
                    let _ = create_window.show();
                    let create_window = create_window.as_weak();
                    let pool = pool.clone();
                    let app = app.clone();
                    let update_weak_window = update_weak_window.clone();
                    create_window
                        .unwrap()
                        .global::<TableData>()
                        .on_create_new_relative({
                            let weak_app = create_window.clone();
                            let pool = pool.clone();
                            let app = app.clone();
                            let update_weak_window = update_weak_window.clone();
                            move |relative, mother_name, father_name| {
                                println!("mothe_name {mother_name} father name {father_name}");
                                create_relative(
                                    relative,
                                    pool.clone(),
                                    father_name.to_string(),
                                    mother_name.to_string(),
                                    weak_app.clone(),
                                    update_weak_window.clone(),
                                    app.clone(),
                                );
                            }
                        });
                }
            });
        }
    });

    update_window.global::<TableData>().on_update_relative({
        let update_weak_window = update_window.as_weak();
        let pool = pool.clone();
        let app = app.as_weak();
        let create_window = CreateWindow::new().unwrap();
        let create_weak_window = create_window.as_weak();
        move |id, relative, mother_name, father_name| {
            update_relative(
                id.to_string(),
                pool.clone(),
                relative,
                mother_name.to_string(),
                father_name.to_string(),
                app.clone(),
                update_weak_window.clone(),
                create_weak_window.clone(),
            );
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
                            let mut selected_mother_name = String::new();
                            let mut selected_father_name = String::new();
                            if let Ok(m_row) =
                                sqlx::query("SELECT full_name FROM relative WHERE id = $1")
                                    .bind(relative.mother_id)
                                    .fetch_one(&pool)
                                    .await
                            {
                                selected_mother_name = m_row.get("full_name");
                            }
                            if let Ok(f_row) =
                                sqlx::query("SELECT full_name FROM relative WHERE id = $1")
                                    .bind(relative.father_id)
                                    .fetch_one(&pool)
                                    .await
                            {
                                selected_father_name = f_row.get("full_name");
                            }
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_active_relative(relative.clone());
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_active_relative(relative.clone());
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_father_name(SharedString::from(
                                    selected_father_name.clone(),
                                ));
                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_mother_name(SharedString::from(
                                    selected_mother_name.clone(),
                                ));
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_father_name(SharedString::from(
                                    selected_father_name.clone(),
                                ));
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_selected_mother_name(SharedString::from(
                                    selected_mother_name.clone(),
                                ));
                        }

                        get_all_files_for_relative(id.to_string(), &pool, weak_app.clone()).await;
                        get_all_notes_for_relative(id.to_string(), &pool, weak_app.clone()).await;
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
            let weak_app = weak_app.clone();
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
                                            .bind(
                                                xx.file_name()
                                                    .unwrap()
                                                    .to_str()
                                                    .unwrap()
                                                    .to_string(),
                                            )
                                            .execute(&pool)
                                            .await;
                                        get_all_files_for_relative(
                                            id.to_string(),
                                            &pool,
                                            weak_app.clone(),
                                        )
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
            let weak_app = weak_app.clone();
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
                            Ok(_) => {
                                get_all_notes_for_relative(id.to_string(), &pool, weak_app.clone())
                                    .await;
                            }
                            Err(e) => {
                                println!("error: {}", e.to_string());
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

    update_window.global::<TableData>().on_delete_relative({
        let weak_app = app.as_weak();
        let pool = pool.clone();
        let weak_update_window = update_window.as_weak();
        let create_window = CreateWindow::new().unwrap();
        let weak_create_window = create_window.as_weak();
        move |id| {
            let pool = pool.clone();
            let weak_app = weak_app.clone();
            let weak_update_window = weak_update_window.clone();
            let weak_create_window = weak_create_window.clone();
            let _ = slint::spawn_local({
                let pool = pool.clone();
                async move {
                    let weak_app = weak_app.clone();
                    let weak_update_window = weak_update_window.clone();
                    let weak_create_window = weak_create_window.clone();
                    let res = sqlx::query("DELETE FROM relative WHERE id=$1")
                        .bind(id.to_string())
                        .execute(&pool)
                        .await;

                    match res {
                        Ok(_) => {
                            let _ = slint::spawn_local({
                                let pool = pool.clone();
                                let app = weak_app.clone();
                                let weak_create_window = weak_create_window.clone();
                                let weak_update_window = weak_update_window.clone();
                                async move {
                                    let _ = update_global_data(
                                        pool,
                                        app,
                                        weak_create_window,
                                        weak_update_window,
                                    )
                                    .await;
                                }
                            });
                            println!("deleted where id is {}", id.to_string());
                        }
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
    let sameness: f32 = row.try_get("sameness").unwrap_or(0.0);
    let sex: String = row.get("sex");
    let hotness: f32 = row.try_get("hotness").unwrap_or(0.0);
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

// function to be called on update relative callback
fn update_relative(
    id: String,
    pool: SqlitePool,
    relative: Relative,
    mother_name: String,
    father_name: String,
    app: slint::Weak<Main>,
    update_weak_window: slint::Weak<UpdateWindow>,
    create_weak_window: slint::Weak<CreateWindow>,
) {
    let pool = pool.clone();
    let update_weak_window = update_weak_window.clone();

    let _ = slint::spawn_local({
        let pool = pool.clone();
        let app = app.clone();
        async move {
            let pool = pool.clone();
            let mut mother_id_db = 0;
            let mut father_id_db = 0;

            if let Ok(m_row) = sqlx::query("select id from relative where full_name = $1")
                .bind(mother_name)
                .fetch_one(&pool)
                .await
            {
                mother_id_db = m_row.try_get("id").unwrap_or(0);
            }

            if let Ok(f_row) = sqlx::query("select id from relative where full_name = $1")
                .bind(father_name)
                .fetch_one(&pool)
                .await
            {
                father_id_db = f_row.try_get("id").unwrap_or(0);
            }
            if mother_id_db > 0 && father_id_db > 0 {
                // update both parents
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
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = create_weak_window.clone();
                            let weak_update_window = update_weak_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_update_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_success(SharedString::from(
                                "Updated, You can close this window!",
                            ));
                    }
                    Err(e) => {
                        update_weak_window
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
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = create_weak_window.clone();
                            let weak_update_window = update_weak_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_update_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_success(SharedString::from(
                                "Updated, You can close this window!",
                            ));
                    }
                    Err(e) => {
                        update_weak_window
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
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = create_weak_window.clone();
                            let weak_update_window = update_weak_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_update_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_success(SharedString::from(
                                "Updated, You can close this window!",
                            ));
                    }
                    Err(e) => {
                        update_weak_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_eror(e.to_string().into());
                    }
                }
            } else if father_id_db <= 0 && mother_id_db <= 0 {
                // update no parent id
                let res = sqlx::query(&sql::update_no_parents())
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
                    .bind(id.to_string())
                    .execute(&pool)
                    .await;
                match res {
                    Ok(_) => {
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = create_weak_window.clone();
                            let weak_update_window = update_weak_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_update_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_success(SharedString::from(
                                "Updated, You can close this window!",
                            ));
                    }
                    Err(e) => {
                        update_weak_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_eror(e.to_string().into());
                        println!("{}", e.to_string());
                    }
                }
            }
        }
    });
}

// function to be called on_create_new_relative callback
fn create_relative(
    relative: Relative,
    pool: SqlitePool,
    selected_father_name: String,
    selected_mother_name: String,
    weak_create_window: slint::Weak<CreateWindow>,
    weak_update_window: slint::Weak<UpdateWindow>,
    app: slint::Weak<Main>,
) {
    let pool = pool.clone();
    let weak_create_window = weak_create_window.clone();

    let _ = slint::spawn_local({
        let pool = pool.clone();
        let app = app.clone();
        async move {
            let pool = pool.clone();
            let mut mother_id_db = 0;
            let mut father_id_db = 0;
            if let Ok(m_row) = sqlx::query("select id from relative where full_name = $1")
                .bind(selected_mother_name)
                .fetch_one(&pool)
                .await
            {
                mother_id_db = m_row.try_get("id").unwrap_or(0);
            }

            if let Ok(f_row) = sqlx::query("select id from relative where full_name = $1")
                .bind(selected_father_name)
                .fetch_one(&pool)
                .await
            {
                father_id_db = f_row.try_get("id").unwrap_or(0);
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
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = weak_create_window.clone();
                            let weak_update_window = weak_update_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        println!("created Successfully");

                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_success(SharedString::from(
                                "Created Successfully. You can Close this window",
                            ));
                    }
                    Err(e) => {
                        println!("error: {e}");
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_error(e.to_string().into());
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
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = weak_create_window.clone();
                            let weak_update_window = weak_update_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_success(SharedString::from(
                                "Created Successfully. You can Close this window",
                            ));
                    }
                    Err(e) => {
                        println!("error: {e}");
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_error(e.to_string().into());
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
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = weak_create_window.clone();
                            let weak_update_window = weak_update_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_success(SharedString::from(
                                "Created Successfully. You can Close this window",
                            ));
                    }
                    Err(e) => {
                        println!("error: {e}");
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_error(e.to_string().into());
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
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            let weak_create_window = weak_create_window.clone();
                            let weak_update_window = weak_update_window.clone();
                            async move {
                                let _ = update_global_data(
                                    pool,
                                    app,
                                    weak_create_window,
                                    weak_update_window,
                                )
                                .await;
                            }
                        });
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_success(SharedString::from(
                                "Created Successfully. You can Close this window",
                            ));
                    }
                    Err(e) => {
                        println!("error: {e}");
                        weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .set_create_error(e.to_string().into());
                    }
                }
            }
        }
    });
}

// function to update global data (singleton)
async fn update_global_data(
    pool: SqlitePool,
    app: slint::Weak<Main>,
    _: slint::Weak<CreateWindow>,
    update_window: slint::Weak<UpdateWindow>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let females = repo::get_female_relatives(&pool).await?;
    let relatives = repo::get_all_relative(&pool).await?;
    let employees = repo::get_all_employees(&pool).await?;
    let males = repo::get_male_relatives(&pool).await?;
    let females2 = repo::get_mothers(&pool).await?;
    update_window
        .unwrap()
        .global::<TableData>()
        .set_females(females.clone().into());
    update_window
        .unwrap()
        .global::<TableData>()
        .set_relative(relatives.clone().into());
    update_window
        .unwrap()
        .global::<TableData>()
        .set_females2(females2.clone().into());
    update_window
        .unwrap()
        .global::<TableData>()
        .set_males(males.clone().into());
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
    Ok(())
}

async fn get_all_notes_for_relative(id: String, pool: &SqlitePool, weak_app: slint::Weak<Main>) {
    if let Ok(rows) = sqlx::query(&sql::get_notes_for_relative())
        .bind(id)
        .fetch_all(pool)
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
    }
}

async fn get_all_files_for_relative(id: String, pool: &SqlitePool, weak_app: slint::Weak<Main>) {
    if let Ok(rows) = sqlx::query(&sql::get_files_for_relative())
        .bind(id)
        .fetch_all(pool)
        .await
    {
        let items = Rc::new(VecModel::default());
        for row in rows {
            let name: String = row.try_get("filename").unwrap_or("".to_string());
            items.push(SharedString::from(name));
        }
        weak_app
            .unwrap()
            .global::<TableData>()
            .set_selected_relative_files(items.clone().into());
    }
}
