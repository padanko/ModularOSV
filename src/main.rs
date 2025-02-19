/*
  __  __           _       _               ____     _____  __      __
 |  \/  |         | |     | |             / __ \   / ____| \ \    / /
 | \  / | ___   __| |_   _| | __ _ _ __  | |  | | | (___    \ \  / / 
 | |\/| |/ _ \ / _` | | | | |/ _` | '__| | |  | |  \___ \    \ \/ /  
 | |  | | (_) | (_| | |_| | | (_| | |    | |__| |  ____) |    \  /   
 |_|  |_|\___/ \__,_|\__,_|_|\__,_|_|     \____/  |_____/      \/    
                                                                     

*/

// ウェブサーバー
use actix_web::{web, App, HttpResponse, Responder, 
                HttpServer, HttpRequest};
use error::{ERR_MSG_SETTING_FILE_NOT_FOUND, ERR_MSG_SQLITE_CONNECT_FAIL};
// テンプレートエンジン
use tera::{Context, Tera};

use thread::generate_user_id;
// 非同期処理用です
use tokio;

// データベース
use sqlx;
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

use std::{arch::x86_64, sync::{Arc, Mutex}};

/////////////////////////////////////////////////
/// SQLコマンドを定義                         ///
/////////////////////////////////////////////////

const SQL_GET_TOPIC_ALL: &str = "SELECT title, topic_id, admin FROM Topics";
const SQL_GET_TOPIC: &str = "SELECT title, topic_id, admin FROM Topics WHERE topic_id = $1";
const SQL_GET_POSTS: &str = "SELECT body, name, ip, timestamp FROM Posts WHERE topic_id = $1";
const SQL_MAKE_POST: &str = "INSERT INTO Posts (body, name, ip, timestamp, topic_id) VALUES ($1, $2, $3, $4, $5)";
const SQL_MAKE_TOPIC: &str = "INSERT INTO Topics (title, topic_id, admin) VALUES ($1, $2, $3)";


// page_から始まる場合はGET
// event_から始まる場合はPOST

// ロングポーリング用

#[derive(Debug, Default, Clone)]
struct PollTrigger {
    topic_id: String,
    count: u64,
}


/////////////////////////////////////////////////
/// 表示                                      ///
/////////////////////////////////////////////////

async fn page_index(tera: web::Data<Tera>) -> impl Responder {

    match setting::get_setting().await {
        Ok(setting) => {
            let mut topics: Vec<thread::Topic> = Vec::new();

            // データベースからスレッド一覧を取得
            let database_url = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;

            match pool {
                Ok(pool) => {
                    match sqlx::query(
                        SQL_GET_TOPIC_ALL,
                    )
                    .fetch_all(&pool).await {
                        Ok(result) => {
                            for row in result {
                                let title: String = row.try_get(0).unwrap();
                                let topicid: String = row.try_get(1).unwrap();
                                topics.push(thread::Topic::new(&title, "TMP", &topicid))
                            }
                        },
                        Err(e) => {
                            error::error(&format!("{}", e));
                        }
                    }
                },
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail)
                }
            }

            // HTMLをレンダリング
            let mut ctx = Context::new();
            ctx.insert("title", &setting.bbs_name);
            ctx.insert("description", &setting.bbs_description_html);
            ctx.insert("topics", &topics);

            let html = tera.render("index.html", &ctx).unwrap_or(String::new());

            // 返す
            HttpResponse::Ok()
                .body(html)
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError()
                .body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
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
                    match sqlx::query(
                        SQL_GET_TOPIC,
                    )
                    .bind(&*topic_id)
                    .fetch_one(&pool).await {
                        Ok(result) => {
                            title = result.try_get_unchecked(0).unwrap_or(String::new());
                        },
                        Err(e) => {
                            error::error(&format!("{}", e));
                        }
                    }

                    // スレッド取得
                    match sqlx::query(
                        SQL_GET_POSTS,
                    )
                    .bind(&*topic_id)
                    .fetch_all(&pool).await {
                        Ok(result) => {
                            for row in result {
                                posts.push(thread::Post {
                                    body: row.try_get(0).unwrap_or(String::new()),
                                    name: row.try_get(1).unwrap_or(String::new()),
                                    ip: row.try_get(2).unwrap_or(String::new()),
                                    date: row.try_get(3).unwrap_or(String::new())
                                })
                            }

                        },
                        Err(e) => {
                            error::error(&format!("{}", e));
                        }
                    }
                },
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail)
                }
            }

            // HTMLをレンダリング
            let mut ctx = Context::new();
            ctx.insert("title", &title);
            ctx.insert("posts", &posts);
            ctx.insert("btn_back", &setting.back_button_label);
            ctx.insert("topic_id", &topic_id.to_string());

            let html = tera.render("topic.html", &ctx).unwrap_or(String::new());

            // 返す
            HttpResponse::Ok()
                .body(html)
        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError()
                .body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }

}



/////////////////////////////////////////////////
/// 投稿                                      ///
/////////////////////////////////////////////////

async fn event_make_topic(form_: web::Form<form::MakeTopicFormData>, req: HttpRequest) -> impl Responder {

    let ip_addr = req.headers().get("X-Forwarded-For")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("Unknown");


    match setting::get_setting().await {
        Ok(setting) => {
            let database_url = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;
            let topic_id = thread::generate_topic_id();
            
            match pool {
                Ok(pool) => {
                    if &form_.body != String::new().as_str() {
                        // トピックを作成
                        let _ = sqlx::query(SQL_MAKE_TOPIC)
                            .bind(thread::post_replace_text(&form_.title))
                            .bind(&topic_id)
                            .bind(thread::generate_user_id(ip_addr))
                            .execute(&pool)
                            .await;

                        let name = &form_.name;
                        let text = &form_.body;

                        let text = &thread::post_replace_text(text);
                        let mut name = &thread::post_replace_text(name);
                        
                        if name == "" {
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

                        

                        return HttpResponse::Ok()
                            .content_type("text/html")
                            .body(format!("{}<br><a href='/topic/{}'>[GO]</a>", setting.bbs_success_make_topic_message, topic_id));
                    } else {
                        return HttpResponse::Ok()
                            .body(setting.bbs_error_message_text_is_empty);
                    }
                }
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail)
                }
            }

        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError()
                .body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
        }
    }

}


async fn event_make_post(
    form_: web::Json<form::MakePostFormData>,
    req: HttpRequest,
    trigger: web::Data<Arc<Mutex<PollTrigger>>>,
) -> impl Responder {

    let ip_addr = req.headers().get("X-Forwarded-For")
    .and_then(|v| v.to_str().ok())
    .unwrap_or("Unknown");

    match setting::get_setting().await {
        Ok(setting) => {
            let database_url = &setting.db_sqlite_file_path;
            let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;
            let topic_id = &form_.topicid;
            
            match pool {
                Ok(pool) => {
                    if &form_.body != String::new().as_str() {
                        let mut name = &form_.name;
                        let text = &form_.body;

                        let text = &thread::post_replace_text(text);

                        if name == "" {
                            name = &setting.default_name
                        }

                        let mut trigger = trigger.lock().unwrap();
                        trigger.topic_id = topic_id.clone();
                        trigger.count += 1;

                        // 投稿を作成
                        let _ = sqlx::query(SQL_MAKE_POST)
                            .bind(text)
                            .bind(name)
                            .bind(generate_user_id(ip_addr))
                            .bind(time::get_now())
                            .bind(&topic_id)
                            .execute(&pool)
                            .await;

                        

                        return HttpResponse::Ok()
                            .body("OK");
                    } else {
                        return HttpResponse::Ok()
                            .body(setting.bbs_error_message_text_is_empty);
                    }
                }
                Err(_) => {
                    error::error(error::ERR_MSG_SQLITE_CONNECT_FAIL);
                    return HttpResponse::InternalServerError()
                        .body(setting.bbs_error_connection_to_database_fail)
                }
            }

        }
        Err(_) => {
            error::error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
            HttpResponse::InternalServerError()
                .body(error::ERR_MSG_SETTING_FILE_NOT_FOUND)
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
                let trigger = trigger.lock().unwrap();
                initial_count = trigger.count;
            }

            while start_time.elapsed() < timeout {
                {
                    let trigger = trigger.lock().unwrap();
                    if trigger.topic_id == *topic_id && trigger.count > initial_count {
                        let database_url: &String = &setting.db_sqlite_file_path;
                        let pool = sqlx::sqlite::SqlitePool::connect(database_url).await;
            
                        let mut posts: Vec<thread::Post> = Vec::new(); 
                        match pool {
                            Ok(pool) => {
                                // データベースから最新の投稿を取得
                                match sqlx::query(
                                    SQL_GET_POSTS,
                                )
                                .bind(&*topic_id)
                                .fetch_all(&pool).await {
                                    Ok(result) => {
                                        for row in result {
                                            posts.push(thread::Post {
                                                body: row.try_get(0).unwrap_or(String::new()),
                                                name: row.try_get(1).unwrap_or(String::new()),
                                                ip: row.try_get(2).unwrap_or(String::new()),
                                                date: row.try_get(3).unwrap_or(String::new())
                                            });
                                        }
                                    },
                                    Err(e) => {
                                        error::error(&format!("{}", e));
                                    }
                                }

                                if let Some(post_latest) = posts.pop() {
                                    return HttpResponse::Ok().json(post_latest);
                                }
                            }
                            Err(_) => {
                                error::error(ERR_MSG_SQLITE_CONNECT_FAIL);
                            }
                        }
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        }
        Err(_) => {
            error::error(ERR_MSG_SETTING_FILE_NOT_FOUND);
        }
    }

    HttpResponse::NoContent().finish()
}



/////////////////////////////////////////////////
/// アプリケーション起動                      ///
/////////////////////////////////////////////////

#[tokio::main]
async fn main() {
    println!("ModularOSV - v0.1.0");
    let trigger = Arc::new(Mutex::new(PollTrigger::default()));
    match setting::get_setting().await {
        Ok(setting) => {
            match Tera::new(&setting.template_folder) {
                Ok(tera) => {
                    if let Ok(server) = HttpServer::new(move || {
                        App::new()
                            .app_data(web::Data::new(tera.clone()))
                            .app_data(web::Data::new(trigger.clone()))
                            // ルーティング
                            .route("/", web::get().to(page_index))
                            .route("/topic/{topic_id}", web::get().to(page_topic))
                            .route("/make/topic", web::post().to(event_make_topic))
                            .route("/make/post", web::post().to(event_make_post))
                            .route("/poll/{opic_id}", web::get().to(poll_posts))

                        })
                        .bind(format!("{}:{}", &setting.server_host, setting.server_port))
                    {
                        match server.run().await {
                            Ok(_) => {},
                            Err(e) => {
                                error::error(&format!("{}", e));
                            }
                        }
                    }
                    else {
                        error::fatal_error(error::ERR_MSG_ADDR_BINDING_FAIL);
                    }
                }
                Err(_) => {
                    error::fatal_error(error::ERR_MSG_TERA_INIT_FAIL);
                }
            }
        }
        Err(e) => {
            println!("{}", e);
            error::fatal_error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
        }
    }

}
