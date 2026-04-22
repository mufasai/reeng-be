<<<<<<< HEAD
use surrealdb::engine::remote::http::{Client, Http};
use surrealdb::Surreal;

#[tokio::main]
async fn main() {
    println!("Connecting...");
    let db = Surreal::new::<Http>("127.0.0.1:9999").await;
    println!("Result: {:?}", db.is_ok());
=======
use surrealdb::engine::remote::http::Http;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Material {
    id: Option<surrealdb::sql::Thing>,
    name: String,
    created_at: Option<serde_json::Value>,
    delivery_note_no: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Http>("surrealdb-production-b201.up.railway.app").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    }).await?;
    db.use_ns("yerico").use_db("project_budget").await?;

    let mut response = db.query("SELECT * FROM materials ORDER BY created_at DESC LIMIT 5").await?;
    let materials: Vec<serde_json::Value> = response.take(0)?;
    
    println!("{}", serde_json::to_string_pretty(&materials)?);
    
    Ok(())
>>>>>>> 691c3482bf6f9b955c729ba1398d34d2f0b11b17
}
