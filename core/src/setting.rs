use std::io::Read;

use tokio::{fs, io::AsyncReadExt};

const SETTING_FILE_PATH: &str = "./ModularOSV-Setting.json";

#[derive(serde::Serialize, serde::Deserialize)]
pub enum UserType {
    Admin,
    Moderator,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    user_type: UserType,
    user_name: String,
    password_hash: String,
}

// 規制ワード
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProhibitedWord {
    pub word: String,
    pub reason: String,
}

// アプリ設定の構造体
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApplicationSetting {
    // 表示に関係するもの
    pub bbs_index_background_image_url: String,
    pub bbs_id: String,
    pub bbs_name: String,
    pub bbs_description_html: String,
    pub bbs_error_message_thread_not_found: String,
    pub bbs_error_message_contains_prohibited_words: String,
    pub bbs_error_message_title_is_empty: String,
    pub bbs_error_message_text_is_empty: String,
    pub bbs_error_internal_server_error: String,
    pub bbs_error_connection_to_database_fail: String,
    pub bbs_success_make_topic_message: String,
    pub back_button_label: String,

    // データベース
    pub db_sqlite_file_path: String,

    // 内部に関係するもの
    pub bbs_prohibited_words: Vec<ProhibitedWord>,
    pub bbs_timestamp_format: String,
    pub template_folder: String,
    pub default_name: String,
    pub server_host: String,
    pub server_port: u16,

    // ファイルアップロード
    pub contents_delivery_path: String,

    // ユーザー
    pub server_user: Vec<User>,

    // ID生成
    pub id_charset: String,
    pub id_length: i32,

    // コマンド
    pub render_command_img: bool,
    pub render_command_video: bool,
    pub render_command_url: bool,
    pub post_pleco_run: bool,

    // モジュール
    pub pleco_script_preprocessing: Vec<String>,
}

// アプリケーションのすべての設定を取得する処理
pub async fn get_setting() -> Result<ApplicationSetting, Box<dyn std::error::Error>> {
    let mut setting_file = fs::File::open(SETTING_FILE_PATH).await?;

    let mut buffer = String::new();
    setting_file.read_to_string(&mut buffer).await?;

    let setting: ApplicationSetting = serde_json::from_str(&buffer)?;

    Ok(setting)
}

// アプリケーションのすべての設定を取得する処理、
// ただし同期処理
pub fn get_setting_sync() -> Result<ApplicationSetting, Box<dyn std::error::Error>> {
    let mut setting_file = std::fs::File::open(SETTING_FILE_PATH)?;

    let mut buffer = String::new();

    setting_file.read_to_string(&mut buffer)?;

    let setting: ApplicationSetting = serde_json::from_str(&buffer)?;

    Ok(setting)
}
