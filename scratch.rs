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
}
