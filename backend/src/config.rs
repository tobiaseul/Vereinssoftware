#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_seconds: u64,
    pub refresh_token_expiry_days: i64,
    pub port: u16,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL required"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET required"),
            jwt_expiry_seconds: std::env::var("JWT_EXPIRY_SECONDS")
                .unwrap_or("900".into()).parse().unwrap(),
            refresh_token_expiry_days: std::env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or("7".into()).parse().unwrap(),
            port: std::env::var("BACKEND_PORT")
                .unwrap_or("3000".into()).parse().unwrap(),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or("http://localhost:5173".into()),
        }
    }
}
