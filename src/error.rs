use std::process;

pub const ERR_MSG_ADDR_BINDING_FAIL: &str = "Address binding failed.";
pub const ERR_MSG_SETTING_FILE_NOT_FOUND: &str = "Application setting file does not exist";
pub const ERR_MSG_TERA_INIT_FAIL: &str = "Initialization of the TERA template engine failed.";
pub const ERR_MSG_SQLITE_CONNECT_FAIL: &str = "Connection to database failed.";

// 致命的なエラーの場合は動作を停止します。
pub fn fatal_error(message: &str) {
    eprintln!("< Fatal Error! >    {}", message);
    eprintln!("Stop operation before it adversely affects the system. Shutdown...");
    process::exit(0);
}

pub fn error(message: &str) {
    eprintln!("< Error! >          {}", message);
}