use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("Database Url Not Found!!!"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT Secret Not Found!!!"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .expect("Port Not Found!!!"),
        }
    }
}
