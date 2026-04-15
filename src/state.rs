use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub type DB = Surreal<Client>;

pub struct AppState {
    pub db: DB,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load environment variables with better error messages
        let surreal_url = std::env::var("SURREAL_URL")
            .map_err(|_| "SURREAL_URL environment variable not set")?;
        let surreal_namespace = std::env::var("SURREAL_NAMESPACE")
            .map_err(|_| "SURREAL_NAMESPACE environment variable not set")?;
        let surreal_database = std::env::var("SURREAL_DATABASE")
            .map_err(|_| "SURREAL_DATABASE environment variable not set")?;
        let surreal_username = std::env::var("SURREAL_USERNAME")
            .map_err(|_| "SURREAL_USERNAME environment variable not set")?;
        let surreal_password = std::env::var("SURREAL_PASSWORD")
            .map_err(|_| "SURREAL_PASSWORD environment variable not set")?;
        
        println!("📋 Environment variables loaded:");
        println!("   SURREAL_URL: {}", surreal_url);
        println!("   SURREAL_NAMESPACE: {}", surreal_namespace);
        println!("   SURREAL_DATABASE: {}", surreal_database);

        // Connect to SurrealDB
        println!("🔌 Connecting to SurrealDB at {}...", surreal_url);
        
        // Determine protocol and construct full URL
        let full_url = if surreal_url.starts_with("http://") || surreal_url.starts_with("https://") {
            surreal_url.clone()
        } else if surreal_url.contains("localhost") || surreal_url.contains("127.0.0.1") {
            format!("http://{}", surreal_url)
        } else if surreal_url.contains("railway.internal") {
            // Railway internal network uses HTTP
            format!("http://{}", surreal_url)
        } else {
            // External URLs default to HTTPS
            format!("https://{}", surreal_url)
        };
        
        println!("🔗 Full connection URL: {}", full_url);
        
        let db: Surreal<Client> = if full_url.starts_with("https://") {
            Surreal::new::<Https>(&full_url).await?
        } else {
            Surreal::new::<Http>(&full_url).await?
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
