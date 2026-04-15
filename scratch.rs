use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Surreal::new::<Mem>(()).await?;
    db.use_ns("test").use_db("test").await?;
    
    let query = r#"
        CREATE test CONTENT {
            id: type::thing('test', '123'),
            val: 'hello'
        };
        CREATE test:456 CONTENT { val: 'world' };
    "#;
    
    let mut response = db.query(query).await?;
    let r1: Vec<Value> = response.take(0)?;
    println!("R1: {:?}", r1);
    let r2: Vec<Value> = response.take(1)?;
    println!("R2: {:?}", r2);
    Ok(())
}
