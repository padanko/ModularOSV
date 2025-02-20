use uuid;
use sqlx;
use sha2::{Sha256, Digest};
use regex::Regex;

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

    pub fn count(&self) -> usize {
        self.contents.len()
    }
}

// XSS対策
fn html_escape(text: &str) -> String{
    text   
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
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

            return id; // IDを返す
        },
        Err(_) => { 
            return String::from("???");
        }
    }

}

pub fn generate_uuid() -> String{
    uuid::Uuid::new_v4().to_string()
}



fn replace_regex(command: regex::Regex, conditions: bool, text: &str, tag: &str, key: &str) -> String {
    if conditions {
        command.replace_all(text, |caps: &regex::Captures| {
            format!("<{} {}='{}'>", tag, key, &caps[1])
        }).to_string()
    } else {
        text.to_string()
    }
}

fn replace_regex_link(command: regex::Regex, conditions: bool, text: &str) -> String {
    if conditions {
        command.replace_all(text, |caps: &regex::Captures| {
            format!("<a href='{}'>{}</a>", &caps[1], &caps[1])
        }).to_string()
    } else {
        text.to_string()
    }
}


pub fn render_commands(text: &str) -> String {
    
    let img_command = regex::Regex::new(r"!Img:&quot;(.+)&quot;").unwrap();
    let url_command = regex::Regex::new(r"!URL:&quot;(.+)&quot;").unwrap();

    let text = &text.replace("\n", "<br>");

    match setting::get_setting_sync() {
        Ok(setting) => {
            
            let text = replace_regex(img_command, setting.render_command_img, text, "img", "src");
            let text = replace_regex_link(url_command, setting.render_command_url, &text);

            return text;

        }
        Err(_) => {
            return text.to_string();
        }
    }
}