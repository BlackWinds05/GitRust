use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub display_name: String,
    pub captcha_token: String,
    pub captcha_input: String,
}
