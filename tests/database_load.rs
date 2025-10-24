//! Database loading test
//!
//! This is the ONLY test that loads the full card database.
//! All other tests should load only the specific cards they need.

use mtg_forge_rs::{loader::AsyncCardDatabase as CardDatabase, Result};
use std::path::PathBuf;

/// Test that the full card database can be loaded successfully
/// This is the ONLY test in the entire test suite that should call eager_load()
#[tokio::test]
async fn test_load_full_card_database() -> Result<()> {
    let cardsfolder = PathBuf::from("cardsfolder");
    if !cardsfolder.exists() {
        // Skip test if cardsfolder doesn't exist
        println!("Skipping full database load test - cardsfolder not present");
        return Ok(());
    }

    println!("Loading full card database from cardsfolder...");
    let card_db = CardDatabase::new(cardsfolder);
    let (loaded, duration) = card_db.eager_load().await?;

    println!("Successfully loaded {} cards in {:?}", loaded, duration);

    // Verify we loaded a reasonable number of cards (should be 30,000+)
    assert!(
        loaded > 30000,
        "Expected to load full database (30,000+ cards), but only loaded {}",
        loaded
    );

    // Verify some known cards can be retrieved
    let mountain = card_db.get_card("Mountain").await?;
    assert!(mountain.is_some(), "Mountain should be in database");
    assert_eq!(mountain.unwrap().name.as_str(), "Mountain");

    let lightning_bolt = card_db.get_card("Lightning Bolt").await?;
    assert!(
        lightning_bolt.is_some(),
        "Lightning Bolt should be in database"
    );
    assert_eq!(lightning_bolt.unwrap().name.as_str(), "Lightning Bolt");

    let grizzly_bears = card_db.get_card("Grizzly Bears").await?;
    assert!(
        grizzly_bears.is_some(),
        "Grizzly Bears should be in database"
    );
    assert_eq!(grizzly_bears.unwrap().name.as_str(), "Grizzly Bears");

    Ok(())
}
