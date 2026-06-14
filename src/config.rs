use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub data_dir: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from: String,
    pub session_secret: String,
    pub jwt_secret: String,
    pub base_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".into())
                .parse()?,
            data_dir: env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()),
            smtp_host: env::var("SMTP_HOST").unwrap_or_default(),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".into())
                .parse()?,
            smtp_user: env::var("SMTP_USER").unwrap_or_default(),
            smtp_pass: env::var("SMTP_PASS").unwrap_or_default(),
            smtp_from: env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@gitrust.local".into()),
            session_secret: env::var("SESSION_SECRET")?,
            jwt_secret: env::var("JWT_SECRET")?,
            base_url: env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
        })
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
