//! Common utilities

use reqwest::blocking::Response;

use super::conf::Config;

/// Send LINE Notify
pub fn send_line_notify_with_image(
    msg: &str,
    img_path: &str,
    conf: Config,
) -> Result<Response, Box<dyn std::error::Error>> {
    let url = "https://notify-api.line.me/api/notify";
    let token = format!("Bearer {}", conf.notification.line_notify_token);
    let token = token.as_str();

    let mut head = reqwest::header::HeaderMap::new();
    let token = reqwest::header::HeaderValue::from_str(token)?;
    head.insert("Authorization", token);

    let form = reqwest::blocking::multipart::Form::new()
        .text("message", msg.to_owned())
        .file("imageFile", img_path)?;

    let client = reqwest::blocking::Client::new();

    let res = client.post(url).headers(head).multipart(form).send()?;
    Ok(res)
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;

    #[test]
    fn notification_test() {
        let paths = crate::module::util::path::dir::create_app_sub_dir();
        let conf = crate::module::util::conf::toml::load(&paths.dir.data);
        let res = send_line_notify_with_image("Rust", "asset/img/pylon_10m.jpg", conf.unwrap());
        assert_eq!(res.unwrap().status(), StatusCode::OK);
    }
}
