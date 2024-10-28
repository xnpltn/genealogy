#![allow(deprecated)]
use native_dialog::FileDialog;
use slint::*;
use sqlx::Row;
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use std::fs;
use std::io;
use std::ops::Deref;
use std::rc::Rc;
use std::{env, u64};

mod repo;
mod sql;
mod utils;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*let mut work_dir = String::from(std::format!(
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
    }*/

    let mut work_dir = String::from("app_data");

    match fs::create_dir(work_dir.clone()) {
        Ok(_) => {
            println!("created work dir");
        }
        Err(e) => {
            if e.kind() != io::ErrorKind::AlreadyExists {
                println!("error: {}", e);
                fs::create_dir("app_data").unwrap();
                work_dir = ".geneapp".to_string();
            } else {
                println!("already exists");
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
    let _ = sqlx::query(&sql::create_tables()).execute(&pool).await?;
    //migrate!("./migrations").run(&pool).await?;
    let app = Main::new()?;

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
        let pool = pool.clone();
        let app = Rc::new(app.as_weak());
        let males = repo::get_male_relatives(&pool).await?;
        let females2 = repo::get_mothers(&pool).await?;
        let images_dir = images_dir.clone();
        move || {
            let update_window = UpdateWindow::new().unwrap();
            let check_svg =
                slint::Image::load_from_path(std::path::Path::new("assets/icons/check-circle.svg"))
                    .unwrap();
            let cross_svg =
                slint::Image::load_from_path(std::path::Path::new("assets/icons/x-circle.svg"))
                    .unwrap();
            update_window
                .global::<TableData>()
                .set_cross_image(cross_svg);
            update_window
                .global::<TableData>()
                .set_check_image(check_svg);

            let _ = slint::spawn_local({
                let pool = pool.clone();
                let update_weak_window = Rc::new(update_window.as_weak());
                async move {
                    let males = repo::get_male_relatives(&pool).await.unwrap();
                    let females2 = repo::get_mothers(&pool).await.unwrap();
                    update_weak_window.unwrap().global::<TableData>().set_females2(females2.into());
                    update_weak_window.unwrap().global::<TableData>().set_males(males.into());
                }
            });
            //update_window.global::<TableData>().on_send_close_request({
            //    let weak_app = update_window.as_weak();
            //    move || {
            //        println!("droping update window");
            //        if weak_app
            //            .unwrap()
            //            .global::<TableData>()
            //            .get_update_eror()
            //            .len()
            //            == 0
            //        {
            //            weak_app.unwrap().hide().unwrap();
            //        }
            //    }
            //});

            update_window.show().unwrap();
            //let update_window = Rc::new(update_window.as_weak());
            let active_relative = app.unwrap().global::<TableData>().get_active_relative();
            update_window
                .global::<TableData>()
                .set_active_relative(active_relative.clone());
            update_window
                .global::<TableData>()
                .set_selected_father_name(
                    app.clone()
                        .unwrap()
                        .global::<TableData>()
                        .get_selected_father_name(),
                );
            update_window
                .global::<TableData>()
                .set_selected_mother_name(
                    app.clone()
                        .unwrap()
                        .global::<TableData>()
                        .get_selected_mother_name(),
                );

            let _ = slint::spawn_local({
                let update_window = Rc::new(update_window.as_weak());
                let id = active_relative.id.clone();
                let pool = pool.clone();
                async move {
                    let mut filename = String::new();
                    if let Ok(f) = sqlx::query(
                        r#"
                            SELECT default_image_id FROM  relative WHERE id=$1;
                        "#,
                    )
                    .bind(id.clone().to_string())
                    .fetch_one(&pool)
                    .await
                    {
                        let image_id: i32 = f.get("default_image_id");
                        println!("id  default is: {image_id}");
                        if image_id  > 0{
                            match sqlx::query("SELECT filename FROM image WHERE id=$1").bind(image_id).fetch_one(&pool).await{
                                Ok(r)=> {
                                    
                                    filename = r.get("filename");
                                },
                                Err(e)=>{
                                    println!("err getting filename: {e}");
                                }

                        }
                        }
                    }
                       else if let Ok(f) = sqlx::query(
                        r#"
                                                SELECT filename 
                                                FROM image 
                                                WHERE relative_id = $1
                                                ORDER BY created_at DESC
                                                LIMIT 1;
                                            "#,
                    )
                    .bind(id.clone().to_string())
                    .fetch_one(&pool)
                    .await
                    {
                        filename = f.try_get("filename").unwrap_or("".to_string());
                    }
                    println!("filename is: {filename}");
                    let image =
                        slint::Image::load_from_path(std::path::Path::new(filename.as_str()));
                    match image {
                        Ok(img) => {
                            update_window
                                .unwrap()
                                .global::<TableData>()
                                .set_active_profile_image(img);
                        }
                        Err(e) => {
                            println!("error loading emage: {}", e.to_string());
                            update_window
                                .unwrap()
                                .global::<CrudMessages>()
                                .set_upload_image_error(slint::format!(
                                    "error loading image: {}",
                                    e.to_string()
                                ));
                        }
                    }
                    if let Ok(images) = repo::get_image_rows_for_relative(id.into(), &pool).await {
                        println!("got images ");
                        update_window
                            .unwrap()
                            .global::<TableData>()
                            .set_images_rows_for_active_relative(images.clone().into());
                    } else {
                        println!("got no images");
                    }
                }
            });

            update_window
                .global::<TableData>()
                .on_current_image_row_change({
                    let weak_app = Rc::new(update_window.as_weak());
                    let pool = pool.clone();
                    move |row| {
                        let table_data = weak_app
                            .unwrap()
                            .global::<TableData>()
                            .get_images_rows_for_active_relative();
                        let table_data = table_data.row_data(row as usize).unwrap();
                        if let Some(filename) = table_data.row_data(0) {
                            println!("{}", filename.text);
                            let image = slint::Image::load_from_path(std::path::Path::new(
                                filename.text.to_string().as_str(),
                            ))
                            .unwrap();

                            weak_app
                                .unwrap()
                                .global::<TableData>()
                                .set_active_profile_image(image);
                            let _ = slint::spawn_local({
                                let weak_app = Rc::clone(&weak_app);
                                let pool = pool.clone();
                                let active_rel = weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .get_active_relative();

                                async move {
                                    match sqlx::query("
                                        UPDATE relative 
                                        SET default_image_id = (SELECT id FROM image WHERE filename = $1)
                                        WHERE id = $2
                                            ")
                                        .bind(filename.clone().text.to_string())
                                        .bind(active_rel.id.to_string())
                                        .execute(&pool)
                                        .await{
                                        Ok(_)=>{println!(" saved default_image_id");},
                                        Err(e)=>{println!("error occured while setting default image {e}");}
                                    }
                                }
                            });
                        }
                    }
                });
            update_window
                .global::<TableData>()
                .on_delete_current_image({
                    let weak_app = Rc::new(update_window.as_weak());
                    let pool = Rc::new(pool.clone());
                    move |row| {
                        let table_data = weak_app
                            .unwrap()
                            .global::<TableData>()
                            .get_images_rows_for_active_relative();
                        let table_data = table_data.row_data(row as usize).unwrap();
                        if let Some(filename) = table_data.row_data(0) {
                            println!("{}", filename.text);
                            let _ = slint::spawn_local({
                                let pool = Rc::clone(&pool);
                                let weak_app = Rc::clone(&weak_app);
                                async move {
                                    let res = sqlx::query("DELETE from image WHERE filename = $1")
                                        .bind(filename.clone().text.to_string())
                                        .execute(pool.deref())
                                        .await;
                                    match res {
                                        Ok(_) => {
                                            if let Ok(images) = repo::get_image_rows_for_relative(
                                                weak_app
                                                    .unwrap()
                                                    .global::<TableData>()
                                                    .get_active_relative()
                                                    .id
                                                    .to_string(),
                                                &pool,
                                            )
                                            .await
                                            {
                                                println!("got images ");
                                                weak_app
                                                    .unwrap()
                                                    .global::<TableData>()
                                                    .set_images_rows_for_active_relative(
                                                        images.clone().into(),
                                                    );

                                                let mut filename = String::new();
                                                if let Ok(f) = sqlx::query(
                                                    r#"
                                                SELECT filename 
                                                FROM image 
                                                WHERE relative_id = $1
                                                ORDER BY created_at DESC
                                                LIMIT 1;
                                            "#,
                                                )
                                                .bind(
                                                    weak_app
                                                        .unwrap()
                                                        .global::<TableData>()
                                                        .get_active_relative()
                                                        .id
                                                        .to_string(),
                                                )
                                                .fetch_one(pool.deref())
                                                .await
                                                {
                                                    filename = f
                                                        .try_get("filename")
                                                        .unwrap_or("".to_string());
                                                }
                                                println!("filename is: {filename}");
                                                let image = slint::Image::load_from_path(
                                                    std::path::Path::new(filename.as_str()),
                                                );
                                                match image {
                                                    Ok(img) => {
                                                        weak_app
                                                            .unwrap()
                                                            .global::<CrudMessages>()
                                                            .set_upload_image_success(
                                                                SharedString::from("Image Updated"),
                                                            );
                                                        weak_app
                                                            .unwrap()
                                                            .global::<TableData>()
                                                            .set_active_profile_image(img);
                                                        if let Ok(images) =
                                                            repo::get_image_rows_for_relative(
                                                                weak_app
                                                                    .unwrap()
                                                                    .global::<TableData>()
                                                                    .get_active_relative()
                                                                    .id
                                                                    .to_string(),
                                                                &pool,
                                                            )
                                                            .await
                                                        {
                                                            println!("got images ");
                                                            weak_app
                                                            .unwrap()
                                                            .global::<TableData>()
                                                            .set_images_rows_for_active_relative(
                                                            images.into(),
                                                    );
                                                        } else {
                                                            println!("got no images");
                                                        }
                                                    }
                                                    Err(e) => {
                                                        println!(
                                                            "error loading emage: {}",
                                                            e.to_string()
                                                        );
                                                        weak_app
                                                            .unwrap()
                                                            .global::<CrudMessages>()
                                                            .set_upload_image_error(
                                                                slint::format!(
                                                                    "error loading image: {}",
                                                                    e.to_string()
                                                                ),
                                                            );
                                                    }
                                                }
                                            } else {
                                                println!("got no images");
                                            }
                                        }
                                        Err(e) => {
                                            println!("{e}");
                                        }
                                    }
                                }
                            });
                        }
                    }
                });
            update_window.global::<TableData>().on_update_relative({
                let app = app.clone();
                let pool = pool.clone();
                let weak_app = Rc::new(update_window.as_weak());
                move |id, relative, mother_name, father_name| {
                    update_relative(
                        id.to_string(),
                        pool.clone(),
                        relative,
                        mother_name.to_string(),
                        father_name.to_string(),
                        app.clone(),
                        weak_app.clone(),
                    );
                }
            });
            update_window
                .global::<TableData>()
                .set_males(males.clone().into());
            update_window
                .global::<TableData>()
                .set_females2(females2.clone().into());
            update_window.global::<TableData>().on_delete_relative({
                let weak_app = Rc::clone(&app);
                let pool = pool.clone();
                let weak_update_window = Rc::new(update_window.as_weak());
                move |id| {
                    println!("deleteding where id = {id}");
                    let pool = pool.clone();
                    let weak_app = Rc::clone(&weak_app);
                    let weak_update_window = weak_update_window.clone();
                    let _ = slint::spawn_local({
                        let pool = pool.clone();
                        async move {
                            let weak_app = Rc::clone(&weak_app);
                            let weak_update_window = weak_update_window.clone();
                            let res = sqlx::query("DELETE FROM relative WHERE id=$1")
                                .bind(id.to_string())
                                .execute(&pool)
                                .await;

                            match res {
                                Ok(_) => {
                                    let _ = slint::spawn_local({
                                        let pool = pool.clone();
                                        async move {
                                            let _ = update_global_data(pool, Rc::clone(&weak_app))
                                                .await;
                                        }
                                    });
                                    println!("deleted where id is {}", id.to_string());
                                    weak_update_window.unwrap().hide().unwrap();
                                }
                                Err(e) => {
                                    println!("error deleting: {e}");
                                    weak_update_window
                                        .unwrap()
                                        .global::<CrudMessages>()
                                        .set_delete_relative_error(slint::format!(
                                            "error deleting relative, {e}"
                                        ));
                                }
                            }
                        }
                    });
                }
            });
            update_window
                .global::<TableData>()
                .on_add_image_for_relative({
                    let images_dir = images_dir.clone();
                    let weak_update_window = Rc::new(update_window.as_weak());
                    let pool = pool.clone();
                    move |id| {
                        let weak_update_window = Rc::clone(&weak_update_window);
                        if id == "" {
                            weak_update_window
                                .unwrap()
                                .global::<CrudMessages>()
                                .set_upload_image_error(SharedString::from(
                                    "No selected relative, click on one row.",
                                ));
                        } else {
                            if let Ok(path) = FileDialog::new()
                                .set_location("~/Desktop")
                                .add_filter("Image", &["jpg", "jpeg", "png"])
                                .show_open_single_file()
                            {
                                let images_dir = images_dir.clone();
                                if let Some(path) = path {
                                    let _ = slint::spawn_local({
                                        let pool = pool.clone();
                                        if let Ok(_) = fs::copy(
                                            path.to_str().unwrap(),
                                            std::format!(
                                                "{images_dir}/{id}-{}",
                                                path.file_name()
                                                    .unwrap()
                                                    .to_str()
                                                    .unwrap()
                                                    .to_string()
                                            ),
                                        ) {
                                        } else {
                                            weak_update_window
                                                .unwrap()
                                                .global::<CrudMessages>()
                                                .set_upload_image_error(SharedString::from(
                                                    "Can't add image",
                                                ));
                                            println!("error adding photo");
                                            return;
                                        };
                                        async move {
                                            match sqlx::query(&sql::add_image_for_relative())
                                                .bind(std::format!(
                                                    "{images_dir}/{id}-{}",
                                                    path.file_name()
                                                        .unwrap()
                                                        .to_str()
                                                        .unwrap()
                                                        .to_string()
                                                ))
                                                .bind(id.to_string())
                                                .execute(&pool)
                                                .await
                                            {
                                                Ok(_) => {
                                                    let mut filename = String::new();
                                                    if let Ok(f) = sqlx::query(
                                                        r#"
                                                SELECT filename 
                                                FROM image 
                                                WHERE relative_id = $1
                                                ORDER BY created_at DESC
                                                LIMIT 1;
                                            "#,
                                                    )
                                                    .bind(id.clone().to_string())
                                                    .fetch_one(&pool)
                                                    .await
                                                    {
                                                        filename = f
                                                            .try_get("filename")
                                                            .unwrap_or("".to_string());
                                                    }
                                                    println!("filename is: {filename}");
                                                    let image = slint::Image::load_from_path(
                                                        std::path::Path::new(filename.as_str()),
                                                    );
                                                    match image {
                                                        Ok(img) => {
                                                            weak_update_window
                                                                .unwrap()
                                                                .global::<CrudMessages>()
                                                                .set_upload_image_success(
                                                                    SharedString::from(
                                                                        "Image Updated",
                                                                    ),
                                                                );
                                                            weak_update_window
                                                                .unwrap()
                                                                .global::<TableData>()
                                                                .set_active_profile_image(img);
                                                            if let Ok(images) =
                                                                repo::get_image_rows_for_relative(
                                                                    weak_update_window
                                                                        .unwrap()
                                                                        .global::<TableData>()
                                                                        .get_active_relative()
                                                                        .id
                                                                        .to_string(),
                                                                    &pool,
                                                                )
                                                                .await
                                                            {
                                                                println!("got images ");
                                                                weak_update_window
                                                            .unwrap()
                                                            .global::<TableData>()
                                                            .set_images_rows_for_active_relative(
                                                            images.into(),
                                                    );
                                                            } else {
                                                                println!("got no images");
                                                            }
                                                        }
                                                        Err(e) => {
                                                            println!(
                                                                "error loading emage: {}",
                                                                e.to_string()
                                                            );
                                                            weak_update_window
                                                                .unwrap()
                                                                .global::<CrudMessages>()
                                                                .set_upload_image_error(
                                                                    slint::format!(
                                                                        "error loading image: {}",
                                                                        e.to_string()
                                                                    ),
                                                                );
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    println!("error: {}", e.to_string());
                                                    weak_update_window
                                                        .unwrap()
                                                        .global::<CrudMessages>()
                                                        .set_upload_image_error(
                                                            SharedString::from("Can't add image"),
                                                        );
                                                }
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                });
        }
    });

    app.global::<TableData>().on_show_add_window({
        let pool = pool.clone();
        let app = Rc::new(app.as_weak());
        let images_dir = Rc::new(images_dir.clone());
        move || {
            let _ = slint::spawn_local({
                let pool = pool.clone();
                let app = Rc::clone(&app);
                let images_dir = Rc::clone(&images_dir);
                async move {
                    let females = repo::get_female_relatives(&pool).await.unwrap();
                    let males = repo::get_male_relatives(&pool).await.unwrap();
                    let females2 = repo::get_mothers(&pool).await.unwrap();
                    let create_window = CreateWindow::new().unwrap();

                    let images_dir = Rc::clone(&images_dir);
                    let check_svg = slint::Image::load_from_path(std::path::Path::new(
                        "assets/icons/check-circle.svg",
                    ))
                    .unwrap();
                    let cross_svg = slint::Image::load_from_path(std::path::Path::new(
                        "assets/icons/x-circle.svg",
                    ))
                    .unwrap();
                    /*create_window.global::<TableData>().on_send_close_request({
                        let weak_app = Rc::new(create_window.as_weak());
                        move || {
                            if weak_app
                                .unwrap()
                                .global::<TableData>()
                                .get_create_error()
                                .len()
                                <= 0
                            {
                                weak_app.unwrap().hide().unwrap();
                            }
                        }
                    });
                    */

                    create_window
                        .global::<TableData>()
                        .on_chose_temporary_image({
                            let weak_app = Rc::new(create_window.as_weak());
                            move || {
                                if let Ok(path) = FileDialog::new()
                                    .set_location("~/Desktop")
                                    .add_filter("Image", &["jpg", "jpeg", "png"])
                                    .show_open_single_file()
                                {
                                    if let Some(path) = path {
                                        if let Ok(image) =
                                            slint::Image::load_from_path(path.as_path())
                                        {
                                            weak_app
                                                .unwrap()
                                                .global::<TableData>()
                                                .set_temporary_image(image);
                                            weak_app
                                                .unwrap()
                                                .global::<TableData>()
                                                .set_temporary_image_path(
                                                    path.as_path().to_str().unwrap().into(),
                                                );
                                        }
                                    }
                                }
                            }
                        });
                    create_window
                        .global::<TableData>()
                        .set_cross_image(cross_svg);
                    create_window
                        .global::<TableData>()
                        .set_check_image(check_svg);
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
                    let pool = pool.clone();
                    let app = app.clone();
                    create_window.global::<TableData>().on_create_new_relative({
                        let weak_app = Rc::new(create_window.as_weak());
                        let pool = pool.clone();
                        let app = Rc::clone(&app);
                        let images_dir = Rc::clone(&images_dir);
                        move |relative, mother_name, father_name| {
                            create_relative(
                                relative,
                                pool.clone(),
                                father_name.to_string(),
                                mother_name.to_string(),
                                weak_app.clone(),
                                Rc::clone(&app),
                                images_dir.clone(),
                            );
                        }
                    });
                }
            });
        }
    });

    app.global::<TableData>().on_current_row_changed({
        let weak_app = Rc::new(app.as_weak());
        let pool = Rc::new(pool.clone());
        move |row| {
            let active_tab = weak_app.unwrap().global::<TableData>().get_active_tab();
            //weak_app
            //    .unwrap()
            //    .global::<TableData>()
            //    .set_active_note(Not);
            if active_tab == ActiveTab::Relative {
                let table_data = weak_app.unwrap().global::<TableData>().get_relative();
                let table_data = table_data.row_data(row as usize).unwrap();
                if let Some(id) = table_data.row_data(0) {
                    let id = id.text;
                    let pool = Rc::clone(&pool);
                    let weak_app = weak_app.clone();

                    let _ = slint::spawn_local(async move {
                        let mut filename = String::new();
                        if let Ok(f) = sqlx::query(
                            r#"
                                SELECT filename 
                                FROM image 
                                WHERE relative_id = $1
                                ORDER BY created_at DESC
                                LIMIT 1;
                            "#,
                        )
                        .bind(id.clone().to_string())
                        .fetch_one(pool.deref())
                        .await
                        {
                            filename = f.try_get("filename").unwrap_or("".to_string());
                        }
                        println!("filename is: {filename}");
                        let image =
                            slint::Image::load_from_path(std::path::Path::new(filename.as_str()));
                        match image {
                            Ok(img) => {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_active_profile_image(img);
                            }
                            Err(e) => {
                                println!("error loading emage: {}", e.to_string());
                            }
                        }

                        if let Ok(relative) = get_relative_data(&id, &pool).await {
                            let mut selected_mother_name = String::new();
                            let mut selected_father_name = String::new();

                            let _ = slint::spawn_local({
                                let pool = pool.clone();
                                let relative = relative.clone();
                                let weak_app = weak_app.clone();
                                println!("id: {}", relative.id);
                                async move {
                                    if let Ok(notes) = repo::get_notes_rows_for_relative(
                                        relative.id.to_string(),
                                        &pool,
                                    )
                                    .await
                                    {
                                        println!("{}", "got notes");
                                        weak_app
                                            .unwrap()
                                            .global::<TableData>()
                                            .set_notes_rows_for_active_relative(
                                                notes.clone().into(),
                                            );
                                        println!("{}", "set notes");
                                    }
                                    if let Ok(files) = repo::get_files_rows_for_relative(
                                        relative.id.to_string(),
                                        &pool,
                                    )
                                    .await
                                    {
                                        println!("got files");
                                        weak_app
                                            .unwrap()
                                            .global::<TableData>()
                                            .set_files_rows_for_active_relative(
                                                files.clone().into(),
                                            );
                                    }
                                }
                            });

                            if let Ok(m_row) =
                                sqlx::query("SELECT full_name FROM relative WHERE id = $1")
                                    .bind(relative.mother_id)
                                    .fetch_one(pool.deref())
                                    .await
                            {
                                selected_mother_name = m_row.get("full_name");
                            }
                            if let Ok(f_row) =
                                sqlx::query("SELECT full_name FROM relative WHERE id = $1")
                                    .bind(relative.father_id)
                                    .fetch_one(pool.deref())
                                    .await
                            {
                                selected_father_name = f_row.get("full_name");
                            }
                            weak_app
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
                        }

                        get_all_files_for_relative(
                            id.to_string(),
                            Rc::clone(&pool),
                            Rc::clone(&weak_app),
                        )
                        .await;
                        get_all_notes_for_relative(
                            id.to_string(),
                            Rc::clone(&pool),
                            Rc::clone(&weak_app),
                        )
                        .await;
                    })
                    .unwrap();
                }
            }
        }
    });

    // NOTE SPECIFIC
    app.global::<TableData>().on_current_note_row_change({
        let weak_app = Rc::new(app.as_weak());
        let pool = Rc::new(pool.clone());
        move |row| {
            let table_data = weak_app
                .unwrap()
                .global::<TableData>()
                .get_notes_rows_for_active_relative();
            let table_data = table_data.row_data(row as usize).unwrap();

            if let Some(id) = table_data.row_data(0) {
                println!("{}", id.text);
                let _ = slint::spawn_local({
                    let pool = Rc::clone(&pool);
                    let weak_app = Rc::clone(&weak_app);
                    async move {
                        let note_row = sqlx::query(
                            "SELECT text, id, pinned, relative_id  FROM  note WHERE id=$1",
                        )
                        .bind(id.text.to_string())
                        .fetch_one(pool.deref())
                        .await
                        .unwrap();
                        let text = note_row.try_get("text").unwrap_or(String::new());
                        let id: i32 = note_row.get("id");
                        let pinned: bool = note_row.get("pinned");
                        let relative_id: i32 = note_row.get("relative_id");
                        let note = Note {
                            text: text.into(),
                            id: slint::format!("{}", id),
                            pinned,
                            relative_id: slint::format!("{}", relative_id),
                        };
                        weak_app
                            .unwrap()
                            .global::<TableData>()
                            .set_active_note(note.into());
                    }
                });
            }
        }
    });

    app.global::<TableData>().on_pin_active_note({
        let weak_app = Rc::from(app.as_weak());
        let pool = Rc::from(pool.clone());
        move |id, command| {
            println!("id to be pinned: {id}");
            if command == Command::PIN {
                let _ = slint::spawn_local({
                    let weak_app = Rc::clone(&weak_app);
                    let pool = Rc::clone(&pool);
                    async move {
                        let res = sqlx::query("UPDATE note SET pinned=1 WHERE id=$1")
                            .bind(id.to_string())
                            .execute(pool.deref())
                            .await;
                        match res {
                            Ok(_) => {
                                // reload noted
                                let relative = weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .get_active_relative();
                                if let Ok(notes) = repo::get_notes_rows_for_relative(
                                    relative.id.to_string(),
                                    &pool,
                                )
                                .await
                                {
                                    weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_notes_rows_for_active_relative(notes.clone().into());
                                }

                                let note_row = sqlx::query(
                                    "SELECT text, id, pinned, relative_id  FROM  note WHERE id=$1",
                                )
                                .bind(id.to_string())
                                .fetch_one(pool.deref())
                                .await
                                .unwrap();
                                let text = note_row.try_get("text").unwrap_or(String::new());
                                let id: i32 = note_row.get("id");
                                let pinned: bool = note_row.get("pinned");
                                let relative_id: i32 = note_row.get("relative_id");
                                let note = Note {
                                    text: text.into(),
                                    id: slint::format!("{}", id),
                                    pinned,
                                    relative_id: slint::format!("{}", relative_id),
                                };
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_active_note(note.into());
                                println!("pinned Successfully");
                            }
                            Err(e) => {
                                println!("error: {}", e.to_string());
                            }
                        }
                    }
                });
            } else if command == Command::UNPIN {
                let _ = slint::spawn_local({
                    let weak_app = Rc::clone(&weak_app);
                    let pool = Rc::clone(&pool);
                    async move {
                        let res = sqlx::query("UPDATE note SET pinned=0 WHERE id=$1")
                            .bind(id.to_string())
                            .execute(pool.deref())
                            .await;
                        match res {
                            Ok(_) => {
                                // reload noted
                                let relative = weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .get_active_relative();
                                if let Ok(notes) = repo::get_notes_rows_for_relative(
                                    relative.id.to_string(),
                                    &pool,
                                )
                                .await
                                {
                                    weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_notes_rows_for_active_relative(notes.clone().into());
                                }
                                let note_row = sqlx::query(
                                    "SELECT text, id, pinned, relative_id  FROM  note WHERE id=$1",
                                )
                                .bind(id.to_string())
                                .fetch_one(pool.deref())
                                .await
                                .unwrap();
                                let text = note_row.try_get("text").unwrap_or(String::new());
                                let id: i32 = note_row.get("id");
                                let pinned: bool = note_row.get("pinned");
                                let relative_id: i32 = note_row.get("relative_id");
                                let note = Note {
                                    text: text.into(),
                                    id: slint::format!("{}", id),
                                    pinned,
                                    relative_id: slint::format!("{}", relative_id),
                                };
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_active_note(note.into());
                                println!("pinned Successfully");
                            }
                            Err(e) => {
                                println!("error: {}", e.to_string());
                            }
                        }
                    }
                });
            }
        }
    });

    app.global::<TableData>().on_delete_active_note({
        let pool = Rc::new(pool.clone());
        let weak_app = Rc::new(app.as_weak());
        move |note_id| {
            println!("id {note_id} needs to be deleted on notes");
            let pool = Rc::clone(&pool);
            let weak_app = Rc::clone(&weak_app);
            let _ = slint::spawn_local({
                async move {
                    let res = sqlx::query("DELETE FROM note WHERE id = $1")
                        .bind(note_id.to_string())
                        .execute(pool.deref())
                        .await;
                    match res {
                        Ok(_) => {
                            // reload noted
                            let relative = weak_app
                                .unwrap()
                                .global::<TableData>()
                                .get_active_relative();
                            if let Ok(notes) =
                                repo::get_notes_rows_for_relative(relative.id.to_string(), &pool)
                                    .await
                            {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_notes_rows_for_active_relative(notes.clone().into());
                            }
                            println!("Delete Note");
                        }
                        Err(e) => {
                            println!("error: {}", e.to_string());
                        }
                    }
                }
            });
        }
    });

    app.global::<TableData>().on_save_edited_note({
        let pool = Rc::new(pool.clone());
        let weak_app = Rc::new(app.as_weak());
        move |id, new_note| {
            println!("edited note: {id} {new_note}");
            let pool = Rc::clone(&pool);
            let weak_app = Rc::clone(&weak_app);
            let _ = slint::spawn_local({
                async move {
                    let res = sqlx::query("UPDATE note SET text = $1 WHERE id = $2")
                        .bind(new_note.to_string())
                        .bind(id.to_string())
                        .execute(pool.deref())
                        .await;
                    match res {
                        Ok(_) => {
                            // reload noted
                            let relative = weak_app
                                .unwrap()
                                .global::<TableData>()
                                .get_active_relative();
                            if let Ok(notes) =
                                repo::get_notes_rows_for_relative(relative.id.to_string(), &pool)
                                    .await
                            {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_notes_rows_for_active_relative(notes.clone().into());
                            }
                            println!("Saved");
                        }
                        Err(e) => {
                            println!("error: {}", e.to_string());
                        }
                    }
                }
            });
        }
    });
    // NOT SPECIFIC END

    // FILES SPECIFIC
    //

    app.global::<TableData>().on_current_file_row_change({
        let weak_app = Rc::new(app.as_weak());
        let pool = Rc::new(pool.clone());
        move |row| {
            let table_data = weak_app
                .unwrap()
                .global::<TableData>()
                .get_files_rows_for_active_relative();
            let table_data = table_data.row_data(row as usize).unwrap();

            if let Some(id) = table_data.row_data(0) {
                println!("file id id: {}", id.text);

                let _ = slint::spawn_local({
                    let pool = Rc::clone(&pool);
                    let weak_app = Rc::clone(&weak_app);
                    async move {
                        let file_row = sqlx::query("SELECT *  FROM  file WHERE id=$1")
                            .bind(id.text.to_string())
                            .fetch_one(pool.deref())
                            .await
                            .unwrap();
                        let file_id: i32 = file_row.get("id");
                        let filename: String =
                            file_row.try_get("filename").unwrap_or(String::new());
                        let file_type: String = file_row.get("type");
                        let pinned: bool = file_row.get("pinned");
                        let relative_id: i32 = file_row.get("relative_id");
                        let file = File {
                            filename: slint::format!("{filename}"),
                            id: slint::format!("{file_id}"),
                            pinned,
                            r#type: slint::format!("{file_type}"),
                            relative_id: slint::format!("{relative_id}"),
                        };
                        println!("file and type: {filename}, {file_type}");
                        weak_app
                            .unwrap()
                            .global::<TableData>()
                            .set_active_file(file.into());
                    }
                });
            }
        }
    });
    app.global::<TableData>().on_delete_active_file({
        let pool = Rc::new(pool.clone());
        let weak_app = Rc::new(app.as_weak());
        move |file_id| {
            println!("id {file_id} needs to be deleted on files");
            let _ = slint::spawn_local({
                let pool = Rc::clone(&pool);
                let weak_app = Rc::clone(&weak_app);
                async move {
                    let res = sqlx::query("DELETE FROM file WHERE id=$1")
                        .bind(file_id.to_string())
                        .execute(pool.deref())
                        .await;
                    match res {
                        Ok(_) => {
                            println!("deleted file Successfully");
                            let relative = weak_app
                                .unwrap()
                                .global::<TableData>()
                                .get_active_relative();
                            if let Ok(files) =
                                repo::get_files_rows_for_relative(relative.id.to_string(), &pool)
                                    .await
                            {
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_files_rows_for_active_relative(files.clone().into());
                            }
                        }
                        Err(e) => {
                            // unlikely to fail
                            println!("{e}");
                        }
                    }
                }
            });
        }
    });

    app.global::<TableData>().on_pin_active_file({
        let pool = Rc::new(pool.clone());
        let weak_app = Rc::new(app.as_weak());
        move |file_id, command| {
            println!("{file_id} needs to be pinned");
            if command == Command::PIN {
                println!(" pinnig file where id = {file_id}");
                let _ = slint::spawn_local({
                    let pool = Rc::clone(&pool);
                    let weak_app = Rc::clone(&weak_app);
                    async move {
                        let res = sqlx::query("UPDATE file SET pinned=1 WHERE id=$1")
                            .bind(file_id.to_string())
                            .execute(pool.deref())
                            .await;
                        match res {
                            Ok(_) => {
                                let relative = weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .get_active_relative();
                                if let Ok(files) = repo::get_files_rows_for_relative(
                                    relative.id.to_string(),
                                    &pool,
                                )
                                .await
                                {
                                    weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_files_rows_for_active_relative(files.clone().into());
                                }
                                let file_row = sqlx::query("SELECT *  FROM  file WHERE id=$1")
                                    .bind(file_id.to_string())
                                    .fetch_one(pool.deref())
                                    .await
                                    .unwrap();
                                let file_id: i32 = file_row.get("id");
                                let filename: String =
                                    file_row.try_get("filename").unwrap_or(String::new());
                                let file_type: String = file_row.get("type");
                                let pinned: bool = file_row.get("pinned");
                                let relative_id: i32 = file_row.get("relative_id");
                                let file = File {
                                    filename: slint::format!("{filename}"),
                                    id: slint::format!("{file_id}"),
                                    pinned,
                                    r#type: slint::format!("{file_type}"),
                                    relative_id: slint::format!("{relative_id}"),
                                };
                                println!("file and type: {filename}, {file_type}");
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_active_file(file.into());
                                println!("pinned Successfully");
                            }
                            Err(e) => {
                                println!("error: {}", e.to_string());
                            }
                        }
                    }
                });
            } else if command == Command::UNPIN {
                println!(" unpinnig file where id = {file_id}");
                let _ = slint::spawn_local({
                    let pool = Rc::clone(&pool);
                    let weak_app = Rc::clone(&weak_app);
                    async move {
                        let res = sqlx::query("UPDATE file SET pinned=0 WHERE id=$1")
                            .bind(file_id.to_string())
                            .execute(pool.deref())
                            .await;
                        match res {
                            Ok(_) => {
                                let relative = weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .get_active_relative();
                                if let Ok(files) = repo::get_files_rows_for_relative(
                                    relative.id.to_string(),
                                    &pool,
                                )
                                .await
                                {
                                    weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_files_rows_for_active_relative(files.clone().into());
                                }

                                let file_row = sqlx::query("SELECT *  FROM  file WHERE id=$1")
                                    .bind(file_id.to_string())
                                    .fetch_one(pool.deref())
                                    .await
                                    .unwrap();
                                let file_id: i32 = file_row.get("id");
                                let filename: String =
                                    file_row.try_get("filename").unwrap_or(String::new());
                                let file_type: String = file_row.get("type");
                                let pinned: bool = file_row.get("pinned");
                                let relative_id: i32 = file_row.get("relative_id");
                                let file = File {
                                    filename: slint::format!("{filename}"),
                                    id: slint::format!("{file_id}"),
                                    pinned,
                                    r#type: slint::format!("{file_type}"),
                                    relative_id: slint::format!("{relative_id}"),
                                };
                                println!("file and type: {filename}, {file_type}");
                                weak_app
                                    .unwrap()
                                    .global::<TableData>()
                                    .set_active_file(file.into());
                                println!("pinned Successfully");
                            }
                            Err(e) => {
                                println!("error: {}", e.to_string());
                            }
                        }
                    }
                });
            }
        }
    });

    //FILES SPECIFIC END
    app.global::<TableData>().set_males(males.clone().into());
    app.global::<TableData>()
        .set_females2(females2.clone().into());

    app.global::<TableData>().on_add_files_for_relative({
        let weak_app = Rc::new(app.as_weak());
        let pool = Rc::new(pool.clone());
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
                                            .execute(pool.deref())
                                            .await;
                                        get_all_files_for_relative(
                                            id.to_string(),
                                            Rc::clone(&pool),
                                            weak_app.clone(),
                                        )
                                        .await;
                                        match res {
                                            Ok(_) => {
                                                if let Ok(files) =
                                                    repo::get_files_rows_for_relative(
                                                        id.to_string(),
                                                        &pool,
                                                    )
                                                    .await
                                                {
                                                    weak_app
                                                        .unwrap()
                                                        .global::<TableData>()
                                                        .set_files_rows_for_active_relative(
                                                            files.clone().into(),
                                                        );
                                                }

                                                println!("created");
                                            }
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
        let weak_app = Rc::new(app.as_weak());
        let pool = Rc::new(pool.clone());
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
                    let pool = Rc::clone(&pool);
                    async move {
                        let res = sqlx::query(&sql::add_note_for_relative())
                            .bind(id.to_string())
                            .bind(note.to_string())
                            .execute(pool.deref())
                            .await;
                        match res {
                            Ok(_) => {
                                if let Ok(notes) =
                                    repo::get_notes_rows_for_relative(id.to_string(), &pool).await
                                {
                                    weak_app
                                        .unwrap()
                                        .global::<TableData>()
                                        .set_notes_rows_for_active_relative(notes.clone().into());
                                }
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
    let hotness: i32 = row.try_get("hotness").unwrap_or(0);
    let crazy: i32 = row.get("crazy");
    let swarthy: i32 = row.get("swarthy");
    let employable: i32 = row.get("employable");
    let mother_id: i32 = row.get("mother_id");
    let father_id: i32 = row.get("father_id");
    let date = utils::from_sqlite_to_date(birthday);

    let relative = Relative {
        id: slint::format!("{id}"),
        first_name: fname.into(),
        last_name: lname.into(),
        middle_name: mname.into(),
        birthday: date.into(),
        email: email.into(),
        lost_reason: lost_reason.into(),
        phone: phone.into(),
        pinned: pinned.into(),
        sameness: slint::format!("{}", sameness),
        sex: sex.into(),
        note: "".into(),
        hotness,
        crazy,
        swarthy,
        employable,
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
    app: Rc<slint::Weak<Main>>,
    update_weak_window: Rc<slint::Weak<UpdateWindow>>,
) {
    let _ = slint::spawn_local({
        async move {
            let mut mother_id_db = 0;
            let mut father_id_db = 0;
            if relative.email.len() > 0 && !utils::is_valid_email(&relative.email.to_string()) {
                update_weak_window
                    .unwrap()
                    .global::<TableData>()
                    .set_update_eror(SharedString::from("Invalid Email Adress"));
                return;
            }

            if relative.phone.len() > 0 && !utils::is_valid_phone(&relative.phone.to_string()) {
                update_weak_window
                    .unwrap()
                    .global::<TableData>()
                    .set_update_eror(SharedString::from("Invalid Phone Number"));
                return;
            }

            if !utils::is_valid_date(&relative.birthday.to_string()) {
                update_weak_window
                    .unwrap()
                    .global::<TableData>()
                    .set_update_eror(SharedString::from("Invalid Date"));
                return;
            }
            let mut sqlite_date = String::new();
            match utils::sqlite_date(relative.birthday.to_string()) {
                Ok(date) => {
                    sqlite_date = date;
                }
                Err(_) => {
                    update_weak_window
                        .unwrap()
                        .global::<TableData>()
                        .set_update_eror(SharedString::from("Invalid Date"));
                    drop(sqlite_date);
                    return;
                }
            }

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
                    .bind(sqlite_date.clone())
                    .bind(relative.first_name.to_string())
                    .bind(relative.middle_name.to_string())
                    .bind(relative.last_name.to_string())
                    .bind(relative.phone.to_string())
                    .bind(relative.email.to_string())
                    .bind(relative.pinned)
                    .bind(father_id_db)
                    .bind(mother_id_db)
                    .bind(relative.employable)
                    .bind(relative.swarthy)
                    .bind(relative.hotness)
                    .bind(relative.crazy)
                    .bind(id.to_string())
                    .execute(&pool)
                    .await;

                match res {
                    Ok(_) => {
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            async move {
                                let _ = update_global_data(pool, Rc::clone(&app)).await;
                            }
                        });
                        //weak_update_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_update_success(SharedString::from(
                        //        "Updated, You can close this window!",
                        //    ));
                        weak_update_window.unwrap().hide().unwrap();
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
                    .bind(sqlite_date.clone())
                    .bind(relative.first_name.to_string())
                    .bind(relative.middle_name.to_string())
                    .bind(relative.last_name.to_string())
                    .bind(relative.phone.to_string())
                    .bind(relative.email.to_string())
                    .bind(relative.pinned)
                    .bind(mother_id_db)
                    .bind(relative.employable)
                    .bind(relative.swarthy)
                    .bind(relative.hotness)
                    .bind(relative.crazy)
                    .bind(id.to_string())
                    .execute(&pool)
                    .await;
                match res {
                    Ok(_) => {
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        //weak_update_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_update_success(SharedString::from(
                        //        "Updated, You can close this window!",
                        //    ));
                        weak_update_window.unwrap().hide().unwrap();
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
                    .bind(sqlite_date.clone())
                    .bind(relative.first_name.to_string())
                    .bind(relative.middle_name.to_string())
                    .bind(relative.last_name.to_string())
                    .bind(relative.phone.to_string())
                    .bind(relative.email.to_string())
                    .bind(relative.pinned)
                    .bind(father_id_db)
                    .bind(relative.employable)
                    .bind(relative.swarthy)
                    .bind(relative.hotness)
                    .bind(relative.crazy)
                    .bind(id.to_string())
                    .execute(&pool)
                    .await;
                match res {
                    Ok(_) => {
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        //weak_update_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_update_success(SharedString::from(
                        //        "Updated, You can close this window!",
                        //    ));
                        weak_update_window.unwrap().hide().unwrap();
                    }
                    Err(e) => {
                        update_weak_window
                            .unwrap()
                            .global::<TableData>()
                            .set_update_eror(e.to_string().into());
                    }
                }
            } else if father_id_db <= 0 && mother_id_db <= 0 {
                // update no parents
                let res = sqlx::query(&sql::update_no_parents())
                    .bind(relative.sameness.to_string())
                    .bind(relative.lost_reason.to_string())
                    .bind(relative.sex.to_string())
                    .bind(sqlite_date.clone())
                    .bind(relative.first_name.to_string())
                    .bind(relative.middle_name.to_string())
                    .bind(relative.last_name.to_string())
                    .bind(relative.phone.to_string())
                    .bind(relative.email.to_string())
                    .bind(relative.pinned)
                    .bind(relative.employable)
                    .bind(relative.swarthy)
                    .bind(relative.hotness)
                    .bind(relative.crazy)
                    .bind(id.to_string())
                    .execute(&pool)
                    .await;
                match res {
                    Ok(_) => {
                        let weak_update_window = update_weak_window.clone();
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = app.clone();
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        //weak_update_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_update_success(SharedString::from(
                        //        "Updated, You can close this window!",
                        //    ));
                        weak_update_window.unwrap().hide().unwrap();
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
    weak_create_window: Rc<slint::Weak<CreateWindow>>,
    app: Rc<slint::Weak<Main>>,
    image_dir: Rc<String>,
) {
    let pool = pool.clone();
    let weak_create_window = weak_create_window.clone();

    let _ = slint::spawn_local({
        let pool = pool.clone();
        let app = Rc::clone(&app);
        async move {
            if relative.email.len() > 0 && !utils::is_valid_email(&relative.email.to_string()) {
                weak_create_window
                    .unwrap()
                    .global::<TableData>()
                    .set_create_error(SharedString::from("Invalid Email Adress"));
                return;
            }

            if relative.phone.len() > 0 && !utils::is_valid_phone(&relative.phone.to_string()) {
                weak_create_window
                    .unwrap()
                    .global::<TableData>()
                    .set_create_error(SharedString::from("Invalid Phone Number"));
                return;
            }

            if !utils::is_valid_date(&relative.birthday.to_string()) {
                weak_create_window
                    .unwrap()
                    .global::<TableData>()
                    .set_create_error(SharedString::from("Invalid Date"));
                return;
            }
            let mut sqlite_date = String::new();
            match utils::sqlite_date(relative.birthday.to_string()) {
                Ok(date) => {
                    sqlite_date = date;
                }
                Err(_) => {
                    weak_create_window
                        .unwrap()
                        .global::<TableData>()
                        .set_create_error(SharedString::from("Invalid Date"));
                    drop(sqlite_date);
                    return;
                }
            }

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
                    .bind(sqlite_date.clone())
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
                    .fetch_one(&pool)
                    .await;
                match res {
                    Ok(row) => {
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = Rc::clone(&app);
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        println!("created Successfully");
                        let id: u32 = row.get("id");
                        println!("created id= {id}");
                        let image_path = weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .get_temporary_image_path()
                            .to_string();
                        println!("path global {image_path}");
                        let file_path = std::path::Path::new(&image_path);
                        if let Some(fname) = file_path.file_name() {
                            if let Some(name) = fname.to_str() {
                                println!("file name is: {name}");
                            }
                        }

                        //weak_create_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_create_success(SharedString::from(
                        //        "Created Successfully. You can Close this window",
                        //    ));
                        weak_create_window.unwrap().hide().unwrap();
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
                    .bind(sqlite_date.clone())
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
                    .fetch_one(&pool)
                    .await;
                match res {
                    Ok(row) => {
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = Rc::clone(&app);
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        println!("created Successfully");
                        let id: u32 = row.get("id");
                        println!("created id= {id}");
                        let image_path = weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .get_temporary_image_path();
                        println!("image path: {image_path}");

                        //weak_create_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_create_success(SharedString::from(
                        //        "Created Successfully. You can Close this window",
                        //    ));
                        weak_create_window.unwrap().hide().unwrap();
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
                    .bind(sqlite_date.clone())
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
                    .fetch_one(&pool)
                    .await;
                match res {
                    Ok(row) => {
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = Rc::clone(&app);
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        println!("created Successfully");
                        let id: u32 = row.get("id");
                        println!("created id= {id}");
                        let image_path = weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .get_temporary_image_path();
                        println!("image path: {image_path}");

                        //weak_create_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_create_success(SharedString::from(
                        //        "Created Successfully. You can Close this window",
                        //    ));
                        weak_create_window.unwrap().hide().unwrap();
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
                    .bind(sqlite_date.clone())
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
                    .fetch_one(&pool)
                    .await;
                match res {
                    Ok(row) => {
                        let _ = slint::spawn_local({
                            let pool = pool.clone();
                            let app = Rc::clone(&app);
                            async move {
                                let _ = update_global_data(pool, app).await;
                            }
                        });
                        println!("created Successfully");
                        let id: u32 = row.get("id");
                        println!("created id= {id}");
                        let image_path = weak_create_window
                            .unwrap()
                            .global::<TableData>()
                            .get_temporary_image_path()
                            .to_string();
                        println!("path global {image_path}");
                        let file_path = std::path::PathBuf::from(&std::format!("{image_path}"));

                        if let Some(name) = file_path.file_name().and_then(|fname| fname.to_str()) {
                            println!("file name is: {name}");
                            let dest_path =
                                std::path::PathBuf::from(&std::format!("{image_dir}/{name}"));

                            if fs::copy(&file_path, &dest_path).is_ok() {
                                let _ = slint::spawn_local({
                                    let query = r#"INSERT INTO image (filename, relative_id) VALUES ($1, $2)"#;
                                    let dest_path_str = dest_path.to_string_lossy().to_string();
                                    async move {
                                        let res = sqlx::query(query)
                                            .bind(dest_path_str)
                                            .bind(id)
                                            .execute(&pool)
                                            .await;
                                        match res {
                                            Ok(_) => {
                                                println!("saved");
                                            }
                                            Err(e) => {
                                                println!("{e}");
                                            }
                                        }
                                    }
                                });
                            }
                        }
                        //weak_create_window
                        //    .unwrap()
                        //    .global::<TableData>()
                        //    .set_create_success(SharedString::from(
                        //        "Created Successfully. You can Close this window",
                        //    ));
                        weak_create_window.unwrap().hide().unwrap();
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
    app: Rc<slint::Weak<Main>>,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let females = repo::get_female_relatives(&pool).await?;
    let relatives = repo::get_all_relative(&pool).await?;
    let employees = repo::get_all_employees(&pool).await?;
    let males = repo::get_male_relatives(&pool).await?;
    let females2 = repo::get_mothers(&pool).await?;
    app.unwrap()
        .global::<TableData>()
        .set_females(females.into());
    app.unwrap()
        .global::<TableData>()
        .set_relative(relatives.into());
    app.unwrap()
        .global::<TableData>()
        .set_employees(employees.into());
    app.unwrap()
        .global::<TableData>()
        .set_females2(females2.into());
    app.unwrap().global::<TableData>().set_males(males.into());
    Ok(())
}

async fn get_all_notes_for_relative(
    id: String,
    pool: Rc<SqlitePool>,
    weak_app: Rc<slint::Weak<Main>>,
) {
    if let Ok(rows) = sqlx::query(&sql::get_notes_for_relative())
        .bind(id)
        .fetch_all(pool.deref())
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
            .set_selected_relative_notes(items.into());
    }
}

async fn get_all_files_for_relative(
    id: String,
    pool: Rc<SqlitePool>,
    weak_app: Rc<slint::Weak<Main>>,
) {
    if let Ok(rows) = sqlx::query(&sql::get_files_for_relative())
        .bind(id)
        .fetch_all(pool.deref())
        .await
    {
        let items = Rc::new(VecModel::default());
        for row in rows {
            let name: String = row.try_get("filename").unwrap_or("".to_string());
            if name.len() > 0 {
                items.push(SharedString::from(name));
            }
        }
        weak_app
            .unwrap()
            .global::<TableData>()
            .set_selected_relative_files(items.into());
    }
}
