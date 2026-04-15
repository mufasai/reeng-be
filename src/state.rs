use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub type DB = Surreal<Client>;

pub struct AppState {
    pub db: DB,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables
        let surreal_url = std::env::var("SURREAL_URL")?;
        let surreal_namespace = std::env::var("SURREAL_NAMESPACE")?;
        let surreal_database = std::env::var("SURREAL_DATABASE")?;
        let surreal_username = std::env::var("SURREAL_USERNAME")?;
        let surreal_password = std::env::var("SURREAL_PASSWORD")?;

        // Connect to SurrealDB
        println!("🔌 Connecting to SurrealDB at {}...", surreal_url);
        let db: Surreal<Client> = if surreal_url.starts_with("http://") {
            Surreal::new::<Http>(&surreal_url).await?
        } else if surreal_url.starts_with("https://") {
            Surreal::new::<Https>(&surreal_url).await?
        } else if surreal_url.contains("localhost") || surreal_url.contains("127.0.0.1") {
            Surreal::new::<Http>(&surreal_url).await?
        } else {
            Surreal::new::<Https>(&surreal_url).await?
        };

        // Sign in
        db.signin(Root {
            username: &surreal_username,
            password: &surreal_password,
        })
        .await?;

        // Select namespace and database
        db.use_ns(&surreal_namespace)
            .use_db(&surreal_database)
            .await?;

        println!("✅ Connected to SurrealDB: {}/{}", surreal_namespace, surreal_database);

        Ok(Self { db })
    }
}
