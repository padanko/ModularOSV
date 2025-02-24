use crate::setting;

fn replace_regex(
    command: regex::Regex,
    conditions: bool,
    text: &str,
    tag: &str,
    key: &str,
) -> String {
    if conditions {
        command
            .replace_all(text, |caps: &regex::Captures| {
                format!("<{} {}='{}'>", tag, key, &caps[1])
            })
            .to_string()
    } else {
        text.to_string()
    }
}

fn replace_regex_link(command: regex::Regex, conditions: bool, text: &str) -> String {
    if conditions {
        command
            .replace_all(text, |caps: &regex::Captures| {
                format!("<a href='{}'>{}</a>", &caps[1], &caps[1])
            })
            .to_string()
    } else {
        text.to_string()
    }
}

pub fn render_commands(text: &str) -> String {
    let img_command = regex::Regex::new("!Img:&quot;(.+)&quot;").unwrap();
    let url_command = regex::Regex::new("!URL:&quot;(.+)&quot;").unwrap();

    let text = &text.replace("\n", "<br>");

    match setting::get_setting_sync() {
        Ok(setting) => {
            let mut text = replace_regex(img_command, setting.render_command_img, text, "img", "src");
            text = replace_regex_link(url_command, setting.render_command_url, &text);

            text
        }
        Err(_) => text.to_string(),
    }
}
