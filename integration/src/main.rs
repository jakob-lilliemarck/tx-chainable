#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("This is the integration test crate for tx_chainable.");
    println!("\nRepositories:");
    println!("- EventsRepository: Manages events with id, name, and jsonb payload");
    println!("- UsersRepository: Manages users with id and name");
    println!("\nTo run the integration tests:");
    println!("1. Set up a PostgreSQL database");
    println!("2. Set DATABASE_URL environment variable");
    println!("3. Run: cargo test --package tx_chainable_integration");
    
    Ok(())
}
