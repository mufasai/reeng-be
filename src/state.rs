use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub type DB = Surreal<Client>;

pub struct AppState {
    pub db: DB,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let surreal_url = std::env::var("SURREAL_URL")
            .map_err(|_| "SURREAL_URL not set")?;
        let surreal_namespace = std::env::var("SURREAL_NAMESPACE")
            .map_err(|_| "SURREAL_NAMESPACE not set")?;
        let surreal_database = std::env::var("SURREAL_DATABASE")
            .map_err(|_| "SURREAL_DATABASE not set")?;
        let surreal_username = std::env::var("SURREAL_USERNAME")
            .map_err(|_| "SURREAL_USERNAME not set")?;
        let surreal_password = std::env::var("SURREAL_PASSWORD")
            .map_err(|_| "SURREAL_PASSWORD not set")?;

        println!("🔌 Connecting to SurrealDB at {}...", surreal_url);

        let db: Surreal<Client> = Surreal::new::<Http>(&surreal_url).await?;

        db.signin(Root {
            username: &surreal_username,
            password: &surreal_password,
        }).await?;

        db.use_ns(&surreal_namespace)
            .use_db(&surreal_database)
            .await?;

        println!("✅ Connected to SurrealDB: {}/{}", surreal_namespace, surreal_database);

        Ok(Self { db })
    }
}
