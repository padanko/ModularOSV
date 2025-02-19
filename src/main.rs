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
                HttpServer};
// テンプレートエンジン
use tera::{Context, Tera};

// 非同期処理用です
use tokio;

// データベース
use sqlx;

// アプリ設定取得用です
mod setting; 

// エラー系処理はここにまとめてます
mod error;

// スレッド
mod thread;

/////////////////////////////////////////////////
/// ルーティングを定義                        ///
/////////////////////////////////////////////////

async fn page_index(tera: web::Data<Tera>) -> impl Responder {

    match setting::get_setting().await {
        Ok(setting) => {

            
         
            let mut ctx = Context::new();
            ctx.insert("title", &setting.bbs_name);
            ctx.insert("description", &setting.bbs_description_html);

            let html = tera.render("index.html", &ctx).unwrap_or(String::new());

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
/// アプリケーション起動                      ///
/////////////////////////////////////////////////

#[tokio::main]
async fn main() {

    
    match setting::get_setting().await {
        Ok(setting) => {
            match Tera::new(&setting.template_folder) {
                Ok(tera) => {
                    if let Ok(server) = HttpServer::new(move || {
                        App::new()
                            .app_data(web::Data::new(tera.clone()))
                            .route("/", web::get().to(page_index))
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
                Err(e) => {
                    println!("{}", e);
                    error::fatal_error(error::ERR_MSG_TERA_INIT_FAILURE);
                }
            }
        }
        Err(_) => {
            error::fatal_error(error::ERR_MSG_SETTING_FILE_NOT_FOUND);
        }
    }

}
