use surrealdb::engine::remote::http::{Client, Http, Https};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

pub type DB = Surreal<Client>;

pub struct AppState {
    pub db: DB,
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔧 Initializing AppState...");
        
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
        
        // Remove protocol if present (SurrealDB client adds it automatically)
        let clean_url = surreal_url
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        
        // Determine which protocol to use
        let use_https = if surreal_url.starts_with("https://") {
            true
        } else if surreal_url.starts_with("http://") {
            false
        } else if clean_url.contains("localhost") || clean_url.contains("127.0.0.1") {
            false // localhost uses HTTP
        } else if clean_url.contains("railway.internal") {
            false // Railway internal network uses HTTP
        } else {
            true // External URLs default to HTTPS
        };
        
        println!("🔗 Connecting with {} to: {}", if use_https { "HTTPS" } else { "HTTP" }, clean_url);
        
        let db: Surreal<Client> = if use_https {
            match Surreal::new::<Https>(clean_url).await {
                Ok(db) => {
                    println!("✅ HTTPS connection established");
                    db
                }
                Err(e) => {
                    eprintln!("❌ Failed to establish HTTPS connection: {}", e);
                    return Err(Box::new(e));
                }
            }
        } else {
            match Surreal::new::<Http>(clean_url).await {
                Ok(db) => {
                    println!("✅ HTTP connection established");
                    db
                }
                Err(e) => {
                    eprintln!("❌ Failed to establish HTTP connection: {}", e);
                    return Err(Box::new(e));
                }
            }
        };

        // Sign in
        println!("🔐 Signing in to database...");
        match db.signin(Root {
            username: &surreal_username,
            password: &surreal_password,
        }).await {
            Ok(_) => println!("✅ Authentication successful"),
            Err(e) => {
                eprintln!("❌ Authentication failed: {}", e);
                return Err(Box::new(e));
            }
        }

        // Select namespace and database
        println!("📂 Selecting namespace and database...");
        match db.use_ns(&surreal_namespace).use_db(&surreal_database).await {
            Ok(_) => println!("✅ Namespace and database selected"),
            Err(e) => {
                eprintln!("❌ Failed to select namespace/database: {}", e);
                return Err(Box::new(e));
            }
        }

        println!("✅ Connected to SurrealDB: {}/{}", surreal_namespace, surreal_database);

        Ok(Self { db })
    }
}
