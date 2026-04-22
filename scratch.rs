use surrealdb::engine::remote::http::{Client, Http};
use surrealdb::Surreal;

#[tokio::main]
async fn main() {
    println!("Connecting...");
    let db = Surreal::new::<Http>("127.0.0.1:9999").await;
    println!("Result: {:?}", db.is_ok());
}
