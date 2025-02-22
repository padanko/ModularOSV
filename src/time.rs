use crate::setting;
use chrono::Local;

pub fn get_now() -> String {
    let mut timestamp_format = String::from("%Y/%m/%d %H:%M:%S");
    if let Ok(setting) = setting::get_setting_sync() {
        timestamp_format = setting.bbs_timestamp_format;
    }
    let datetime = Local::now().format(&timestamp_format).to_string();

    datetime
}
