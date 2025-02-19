use uuid;
use sqlx;
use sha2::{Sha256, Digest};

use crate::setting;

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Post {
    pub name: String,
    pub body: String,
    pub date: String,
    pub ip: String,
}

#[derive(sqlx::FromRow, serde::Serialize, serde::Deserialize)]
pub struct Topic {
    pub title: String,
    pub thread_admin: String,
    pub topic_id: String,
    pub contents: Vec<Post>,

}

// implにします(将来使うかもしれないので)
impl Topic {
    pub fn new(title: &str, admin: &str, topic_id: &str) -> Topic {
        Self { 
            title: title.to_string(),
            thread_admin: admin.to_string(),
            topic_id: topic_id.to_string(),
            contents: vec![]
        }
    }
}


fn html_escape(text: &str) -> String{
    text   
        .replace("<", "&lt;")
        .replace(">", "&lt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
        .replace("&", "&amp;")
}

// 投稿時に呼び出される変換
pub fn post_replace_text(text: &str) -> String{

    html_escape(text) // XSS対策

}

// IPアドレスを素にIDを生成する
pub fn generate_user_id(ipaddr_: &str) -> String {
    

    match setting::get_setting_sync() {
        Ok(setting) => {

            let charset = setting.id_charset.as_bytes();
            let length = setting.id_length;
        
            let ipaddr: Vec<&str> = ipaddr_.rsplitn(2, ":").collect();
            let mut hasher = Sha256::new();
            hasher.update(ipaddr.get(1).unwrap_or(&String::from("FUCK").as_str()).as_bytes());
            let result = hasher.finalize();
            let id: String = result
                .iter()
                .map(|&byte| {
                    let index = (byte as usize) % charset.len();
                    charset[index] as char
                })
                .take(length as usize)
                .collect();

            return id; // IDを返す
        },
        Err(_) => { 
            return String::from("???");
        }
    }

}

pub fn generate_topic_id() -> String{
    uuid::Uuid::new_v4().to_string()
}