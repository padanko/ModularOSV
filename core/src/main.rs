/*
  __  __           _       _               ____     _____  __      __
 |  \/  |         | |     | |             / __ \   / ____| \ \    / /
 | \  / | ___   __| |_   _| | __ _ _ __  | |  | | | (___    \ \  / /
 | |\/| |/ _ \ / _` | | | | |/ _` | '__| | |  | |  \___ \    \ \/ /
 | |  | | (_) | (_| | |_| | | (_| | |    | |__| |  ____) |    \  /
 |_|  |_|\___/ \__,_|\__,_|_|\__,_|_|     \____/  |_____/      \/


*/

// ウェブサーバー
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};

// ファイルシステム
use actix_multipart::form::MultipartForm;

// テンプレートエンジン
use tera::{Context, Tera};

use thread::generate_user_id;
// 非同期処理用です
use tokio::{self, io::AsyncWriteExt};

// データベース
use sqlx::Row;

// アプリ設定取得用です
mod setting;

// エラー系処理はここにまとめてます
mod error;

// スレッド
mod thread;

// POSTメソッドで受け取るデータの構造体
mod form;

// 時間関係
mod time;

// 拡張用
mod pleco;

// モジュール
mod module;

// テキストデータの加工
mod text;

use std::{io::Read, sync::Arc};
use tokio::sync::Mutex;

///////////////////////////////////////////////
// SQLコマンドを定義                         //
///////////////////////////////////////////////

const SQL_GET_TOPIC_ALL: &str = "SELECT title, topic_id, admin FROM Topics";
const SQL_GET_TOPIC: &str = "SELECT title, topic_id, admin FROM Topics WHERE topic_id = $1";
const SQL_GET_POSTS: &str = "SELECT body, name, ip, timestamp FROM Posts WHERE topic_id = $1";
const SQL_MAKE_POST: &str =
    "INSERT INTO Posts (body, name, ip, timestamp, topic_id) VALUES ($1, $2, $3, $4, $5)";
const SQL_MAKE_TOPIC: &str = "INSERT INTO Topics (title, topic_id, admin) VALUES ($1, $2, $3)";
const SQL_GET_POST_SEARCH: &str = "SELECT body, name, ip, timestamp FROM Posts WHERE body LIKE $1";

// page_から始まる場合はGET
// event_から始まる場合はPOST

// ロングポーリング用

#[derive(Debug, Default, Clone)]
struct PollTrigger {
    topic_id: String,
    count: u64,
}

///////////////////////////////////////////////
// 表示                                      //
///////////////////////////////////////////////

async fn page_index(tera: web::Data<Tera>) -> impl Responder {
    match setting::get_setting().await {
        Ok(setting) => {
            let mut topics: Vec<thread::Topic> = Vec::new();

            // データベースからスレッド一覧を取得
            let database_url = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;

            match pool {
                Ok(pool) => match sqlx::query(SQL_GET_TOPIC_ALL).fetch_all(&pool).await {
                    Ok(result) => {
                        for row in result {
                            let title: String = row.try_get(0).unwrap();
                            let topicid: String = row.try_get(1).unwrap();
                            topics.push(thread::Topic::new(&title, "TMP", &topicid))
                        }
                    }
                    Err(e) => {
                        error::error(&e.to_string());
                    }
                },
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail);
                }
            }

            topics.reverse();

            // HTMLをレンダリング
            let mut ctx = Context::new();
            ctx.insert("title", &setting.bbs_name);
            ctx.insert("description", &setting.bbs_description_html);
            ctx.insert("topics", &topics);
            ctx.insert("background_image_url", &setting.bbs_index_background_image_url);

            let html = tera.render("index.html", &ctx).unwrap_or_default();

            // 返す
            HttpResponse::Ok().body(html)
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }
}

async fn page_topic(topic_id: web::Path<String>, tera: web::Data<Tera>) -> impl Responder {
    match setting::get_setting().await {
        Ok(setting) => {
            let mut title = String::new();
            let mut posts: Vec<thread::Post> = Vec::new();

            let database_url: &String = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;

            match pool {
                Ok(pool) => {
                    // タイトル取得
                    match sqlx::query(SQL_GET_TOPIC)
                        .bind(&*topic_id)
                        .fetch_one(&pool)
                        .await
                    {
                        Ok(result) => {
                            title = result.try_get_unchecked(0).unwrap_or(String::new());
                        }
                        Err(e) => {
                            error::error(&e.to_string());
                        }
                    }

                    // スレッド取得
                    match sqlx::query(SQL_GET_POSTS)
                        .bind(&*topic_id)
                        .fetch_all(&pool)
                        .await
                    {
                        Ok(result) => {
                            for row in result {
                                posts.push(thread::Post {
                                    body: text::render_commands(
                                        &row.try_get(0).unwrap_or(String::new()),
                                    ),
                                    name: row.try_get(1).unwrap_or(String::new()),
                                    ip: row.try_get(2).unwrap_or(String::new()),
                                    date: row.try_get(3).unwrap_or(String::new()),
                                })
                            }
                        }
                        Err(e) => {
                            error::error(&e.to_string());
                        }
                    }
                }
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail);
                }
            }

            // HTMLをレンダリング
            let mut ctx = Context::new();
            ctx.insert("title", &title);
            ctx.insert("posts", &posts);
            ctx.insert("btn_back", &setting.back_button_label);
            ctx.insert("topic_id", &topic_id.to_string());

            let html = tera.render("topic.html", &ctx).unwrap_or_default();

            // 返す
            HttpResponse::Ok().body(html)
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }
}

///////////////////////////////////////////////
// 検索                                      //
///////////////////////////////////////////////

async fn post_search(query: web::Query<form::PostSearchQuery>, tera: web::Data<Tera>) -> impl Responder {

    match setting::get_setting().await {
        Ok(setting) => {
            let mut posts: Vec<thread::Post> = Vec::new();

            let database_url: &String = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;

            match pool {
                Ok(pool) => {
                    // スレッド取得
                    match sqlx::query(SQL_GET_POST_SEARCH)
                        .bind(&format!("%{}%", &*query.query))
                        .fetch_all(&pool)
                        .await
                    {
                        Ok(result) => {
                            for row in result {
                                posts.push(thread::Post {
                                    body: text::render_commands(
                                        &row.try_get(0).unwrap_or(String::new()),
                                    ),
                                    name: row.try_get(1).unwrap_or(String::new()),
                                    ip: row.try_get(2).unwrap_or(String::new()),
                                    date: row.try_get(3).unwrap_or(String::new()),
                                })
                            }
                        }
                        Err(e) => {
                            error::error(&e.to_string());
                        }
                    }
                }
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail);
                }
            }

            // HTMLをレンダリング
            let mut ctx = Context::new();
            ctx.insert("posts", &posts);
            ctx.insert("query", &*query.query);
            ctx.insert("btn_back", &setting.back_button_label);

            match tera.render("post_search_result.html", &ctx) {
                Ok(html) => {
                    HttpResponse::Ok().body(html)
                }, Err(e) => {
                    HttpResponse::InternalServerError().body(e.to_string())
                }
            }

            // 返す
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }

}

///////////////////////////////////////////////
// 投稿                                      //
///////////////////////////////////////////////

async fn event_make_topic(
    form_: web::Form<form::MakeTopicFormData>,
    req: HttpRequest,
) -> impl Responder {
    let ip_addr = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");

    match setting::get_setting().await {
        Ok(setting) => {
            // NGワードの処理
            for prohibited_word in setting.bbs_prohibited_words {
                if form_.body.contains(&prohibited_word.word) {
                    return HttpResponse::Ok().body(format!(
                        "{}\n【{}】\n{}",
                        setting.bbs_error_message_contains_prohibited_words,
                        prohibited_word.word,
                        prohibited_word.reason
                    ));
                }
            }

            let database_url = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;
            let topic_id = thread::generate_uuid();

            match pool {
                Ok(pool) => {
                    if !form_.body.is_empty() {
                        // トピックを作成
                        let _ = sqlx::query(SQL_MAKE_TOPIC)
                            .bind(thread::post_replace_text(&form_.title))
                            .bind(&topic_id)
                            .bind(thread::generate_user_id(ip_addr))
                            .execute(&pool)
                            .await;

                        let name = &form_.name;
                        let text = &form_.body;

                        let text = &thread::post_replace_text_form(text);
                        let mut name = &thread::post_replace_text(name);

                        if name.is_empty() {
                            name = &setting.default_name
                        }

                        // 投稿を作成
                        let _ = sqlx::query(SQL_MAKE_POST)
                            .bind(text)
                            .bind(name)
                            .bind(generate_user_id(ip_addr))
                            .bind(time::get_now())
                            .bind(&topic_id)
                            .execute(&pool)
                            .await;

                        HttpResponse::Ok().content_type("text/html").body(format!(
                            "<meta charset='UTF-8'>{}<br><a href='/topic/{}'>[GO]</a>",
                            setting.bbs_success_make_topic_message, topic_id
                        ))
                    } else {
                        HttpResponse::Ok().body(setting.bbs_error_message_text_is_empty)
                    }
                }
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail)
                }
            }
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }
}

async fn event_make_post(
    form_: web::Json<form::MakePostFormData>,
    req: HttpRequest,
    trigger: web::Data<Arc<Mutex<PollTrigger>>>,
) -> impl Responder {
    let ip_addr = req
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("Unknown");


    match setting::get_setting().await {
        Ok(setting) => {
            
            // NGワードの処理
            for prohibited_word in setting.bbs_prohibited_words {
                if form_.body.contains(&prohibited_word.word) {
                    return HttpResponse::Ok().body(format!(
                        "{}\n【{}】\n{}",
                        setting.bbs_error_message_contains_prohibited_words,
                        prohibited_word.word,
                        prohibited_word.reason
                    ));
                }
            }

            let database_url = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;
            let topic_id = &form_.topicid;

            match pool {
                Ok(pool) => {
                    if !form_.body.is_empty() {
                        let name = &form_.name;
                        let text = &form_.body;

                        let text = &thread::post_replace_text_form(text);
                        let mut name = &thread::post_replace_text(name);

                        if name.is_empty() {
                            name = &setting.default_name
                        }

                    

                        let mut trigger = trigger.lock().await;
                        trigger.topic_id = topic_id.clone();
                        trigger.count += 1;

                        // 投稿を作成
                        let _ = sqlx::query(SQL_MAKE_POST)
                            .bind(text)
                            .bind(name)
                            .bind(generate_user_id(ip_addr))
                            .bind(time::get_now())
                            .bind(topic_id)
                            .execute(&pool)
                            .await;

                        HttpResponse::Ok().body("OK")
                    } else {
                        HttpResponse::Ok().body(setting.bbs_error_message_text_is_empty)
                    }
                }
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail)
                }
            }
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }
}

async fn poll_posts(
    topic_id: web::Path<String>,
    trigger: web::Data<Arc<Mutex<PollTrigger>>>,
) -> impl Responder {
    let timeout = tokio::time::Duration::from_secs(30); // 30秒間待機
    let start_time = tokio::time::Instant::now();
    let initial_count;

    match setting::get_setting().await {
        Ok(setting) => {
            {
                let trigger = trigger.lock().await;
                initial_count = trigger.count;
            }

            while start_time.elapsed() < timeout {
                {
                    let trigger = trigger.lock().await;
                    if trigger.topic_id == *topic_id && trigger.count > initial_count {
                        let database_url: &String = &setting.db_sqlite_file_path;
                        let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;

                        let mut posts: Vec<thread::Post> = Vec::new();
                        match pool {
                            Ok(pool) => {
                                // データベースから最新の投稿を取得
                                match sqlx::query(SQL_GET_POSTS)
                                    .bind(&*topic_id)
                                    .fetch_all(&pool)
                                    .await
                                {
                                    Ok(result) => {
                                        for row in result {
                                            posts.push(thread::Post {
                                                body: text::render_commands(
                                                    &row.try_get(0).unwrap_or(String::new()),
                                                ),
                                                name: row.try_get(1).unwrap_or_default(),
                                                ip: row.try_get(2).unwrap_or_default(),
                                                date: row.try_get(3).unwrap_or_default(),
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        error::error(&e.to_string());
                                    }
                                }

                                if let Some(post_latest) = posts.pop() {
                                    return HttpResponse::Ok().json(post_latest);
                                }
                            }
                            Err(_) => {
                                error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                            }
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
        }
    }

    HttpResponse::NoContent().finish()
}

///////////////////////////////////////////////
// ファイル系                                //
///////////////////////////////////////////////

async fn file_upload(
    payload: MultipartForm<form::UploadForm>,
    tera: web::Data<Tera>,
) -> impl Responder {
    match setting::get_setting().await {
        Ok(setting) => {
            let file_id = thread::generate_uuid();

            let mut file_ = Vec::new();

            let _ = payload.file.file.as_file().read_to_end(&mut file_);

            let filename = payload.file.file_name.clone().unwrap_or_default();
            let filepath = std::path::Path::new(&filename);

            let contents_delivery_path = &setting.contents_delivery_path;

            let ext = match filepath.extension() {
                Some(ext_) => ext_.to_str().unwrap_or_default().to_string(),
                None => String::new(),
            };

            let basename = match filepath.file_stem() {
                Some(basename) => basename.to_str().unwrap_or_default().to_string().replace("..", "").replace("/",""),
                None => String::new(),
            };

            match tokio::fs::File::create(format!(
                "{}/{}_{}.{}",
                contents_delivery_path, &file_id, &basename, &ext
            ))
            .await
            {
                Ok(mut file) => {
                    let _ = file.write_all(&file_).await;
                }
                Err(_) => {
                    error::error(error::ERR_MSG_FILE_UPLOAD_FAIL);
                }
            }

            let mut context = tera::Context::new();

            context.insert("file_id", &format!("/Files/{}_{}.{}", &file_id, &basename, &ext));
            match tera.render("uploaded.html", &context) {
                Ok(html) => HttpResponse::Ok().body(html),
                Err(_) => HttpResponse::InternalServerError().body(error::ERR_MSG_TERA_INIT_FAIL),
            }
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }
}


async fn file_search(
    query: web::Form<form::FileSearchFormData>,
    tera: web::Data<Tera>,
) -> impl Responder {
    match setting::get_setting().await {
        Ok(setting) => {
            let mut file_list: Vec<String> = Vec::new();
            match std::fs::read_dir(setting.contents_delivery_path) {
                Ok(dir) => {
                    for entry in dir {
                        let entry = entry.unwrap();
                        let path = entry.path();
                
                        if path.is_file() {
                            if let Some(file_name) = path.file_name() {
                                let file_name = file_name.to_str().unwrap();
                                if file_name.contains(&query.query) {
                                    file_list.push(file_name.to_string());
                                }
                            }
                        }
                    }

                    let mut ctx = Context::new();
                    
                    ctx.insert("file_list", &file_list);

                    let html = tera.render("file_search_result.html", &ctx).unwrap_or_default();

                    return HttpResponse::Ok().body(html);

                },
                Err(_) => {
                    return HttpResponse::InternalServerError().body(setting.bbs_error_internal_server_error);
                }
            };
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            return HttpResponse::InternalServerError().body(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
        }
    }
}

/////////////////////////////////////////////////
// アプリケーション起動                      //
/////////////////////////////////////////////////

#[tokio::main]
async fn main() {
    println!("ModularOSV - v0.1.5");
    println!("===================");
    println!("BUILD        {}", env!("BUILD_ID"));
    println!("");
    let trigger = Arc::new(Mutex::new(PollTrigger::default()));
    match setting::get_setting().await {
        Ok(setting) => {
            match Tera::new(&setting.template_folder) {
                Ok(tera) => {
                    if let Ok(server) = HttpServer::new(move || {
                        App::new()
                            .app_data(web::Data::new(tera.clone()))
                            .app_data(web::Data::new(trigger.clone()))
                            .service(
                                actix_files::Files::new("/Files", &setting.contents_delivery_path)
                                    .show_files_listing()
                            )
                            // ルーティング
                            .route("/", web::get().to(page_index))
                            .route("/topic/{topic_id}", web::get().to(page_topic))
                            .route("/make/topic", web::post().to(event_make_topic))
                            .route("/make/post", web::post().to(event_make_post))
                            .route("/poll/{opic_id}", web::get().to(poll_posts))
                            .route("/utils/fileupload", web::post().to(file_upload))
                            .route("/utils/filesearch", web::post().to(file_search))
                            .route("/utils/postsearch", web::get().to(post_search))
                    })
                    .bind(format!("{}:{}", &setting.server_host, setting.server_port))
                    {
                        match server.run().await {
                            Ok(_) => {}
                            Err(e) => {
                                error::error(&e.to_string());
                            }
                        }
                    } else {
                        error::fatal_error(error::ERR_MSG_ADDR_BINDING_FAIL);
                    }
                }
                Err(_) => {
                    error::fatal_error(error::ERR_MSG_TERA_INIT_FAIL);
                }
            }
        }
        Err(_) => {
            error::fatal_error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
        }
    }
}
