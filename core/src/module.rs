use crate::pleco;
use crate::setting;


// 設定からPLECoスクリプトを取得しテキストを実行する
pub fn pleco_processing(text: &str) -> String {
    match setting::get_setting_sync() {
        Ok(setting_) => {
            let pleco_ = pleco::pleco::PLECo::new();
            
            pleco_.insert_var("post-text",pleco::lexer::Token::String(text.to_string()));

            let pleco_com = setting_.pleco_script_preprocessing.join("\n");
            pleco_.handle_command(&pleco_com)
        },
        Err(_) => {
            text.to_string()
        } 
    }
}