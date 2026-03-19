use keyring::Entry;

const SERVICE_NAME: &str = "com.disperpusip.playbackannouncer";
const ACCOUNT_NAME: &str = "elevenlabs-api-key";

#[test]
fn test_keychain_save_and_retrieve() {
    let test_key = "sk_test_1234567890abcdef";
    
    // Step 1: Delete any existing entry
    println!("Step 1: Deleting any existing entry...");
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).expect("Failed to create entry");
    let _ = entry.delete_credential(); // Ignore if doesn't exist
    
    // Step 2: Verify it's gone
    println!("Step 2: Verifying entry is deleted...");
    match entry.get_password() {
        Ok(_) => panic!("Entry still exists after delete!"),
        Err(keyring::Error::NoEntry) => println!("✓ Entry confirmed deleted"),
        Err(e) => panic!("Unexpected error checking deletion: {:?}", e),
    }
    
    // Step 3: Save the key
    println!("Step 3: Saving test key...");
    entry.set_password(test_key).expect("Failed to save password");
    println!("✓ Password saved");
    
    // Step 4: Read it back with the SAME entry object
    println!("Step 4: Reading back with same Entry object...");
    match entry.get_password() {
        Ok(retrieved) => {
            println!("✓ Retrieved: {} (length: {})", 
                if retrieved == test_key { "MATCHES" } else { "MISMATCH!" },
                retrieved.len()
            );
            assert_eq!(retrieved, test_key, "Key mismatch with same Entry object!");
        },
        Err(e) => panic!("Failed to retrieve with same Entry object: {:?}", e),
    }
    
    // Step 5: Create a NEW entry object and read again
    println!("Step 5: Creating NEW Entry object and reading...");
    let entry2 = Entry::new(SERVICE_NAME, ACCOUNT_NAME).expect("Failed to create second entry");
    match entry2.get_password() {
        Ok(retrieved) => {
            println!("✓ Retrieved: {} (length: {})", 
                if retrieved == test_key { "MATCHES" } else { "MISMATCH!" },
                retrieved.len()
            );
            assert_eq!(retrieved, test_key, "Key mismatch with new Entry object!");
        },
        Err(e) => panic!("Failed to retrieve with new Entry object: {:?}", e),
    }
    
    // Cleanup
    println!("Cleanup: Deleting test entry...");
    entry2.delete_credential().expect("Failed to cleanup");
    println!("✓ Test complete!");
}
