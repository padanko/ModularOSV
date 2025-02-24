use crate::pleco;
use crate::setting;
use crate::module;

use sha2::{Digest, Sha256};

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
            contents: vec![],
        }
    }
}

// XSS対策
fn html_escape(text: &str) -> String {
    text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

// 投稿時に呼び出される変換
pub fn post_replace_text(text: &str) -> String {
    let mut pleco_run = false;

    match setting::get_setting_sync() {
        Ok(setting_) => { pleco_run = setting_.post_pleco_run },
        Err(_) => { }
    }


    // PLECoScriptが有効かどうか
    if text.starts_with("#PLECoScript#\n") && pleco_run {
        let pleco_object = pleco::pleco::PLECo::new();
        let text_ = pleco_object.handle_command(text);
        let text_ = &html_escape(&text_); // XSS対策
        text_.to_string()
    } else {
        let text_ = &html_escape(text); // XSS対策
        text_.to_string()
    }



}

pub fn post_replace_text_form(text: &str) -> String {

    let text = &module::pleco_processing(text);
    
    post_replace_text(text)

}

// IPアドレスを素にIDを生成する
pub fn generate_user_id(ipaddr_: &str) -> String {
    match setting::get_setting_sync() {
        Ok(setting) => {
            let charset = setting.id_charset.as_bytes();
            let length = setting.id_length;

            let mut hasher = Sha256::new();
            hasher.update(ipaddr_);
            let result = hasher.finalize();
            let id: String = result
                .iter()
                .map(|&byte| {
                    let index = (byte as usize) % charset.len();
                    charset[index] as char
                })
                .take(length as usize)
                .collect();

            id // IDを返す
        }
        Err(_) => String::from("???"),
    }
}

pub fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
