use keyring::Entry;
use std::sync::Mutex;
use once_cell::sync::Lazy;

const SERVICE_NAME: &str = "com.disperpusip.playbackannouncer";
const ACCOUNT_NAME: &str = "elevenlabs-api-key";

// In-memory cache to prevent repeated keychain dialogs
static API_KEY_CACHE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn save_api_key(key: &str) -> Result<(), String> {
    println!("Attempting to save API key to keychain (service: {}, account: {})", SERVICE_NAME, ACCOUNT_NAME);
    
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| {
            let err_msg = format!("Failed to create keyring entry: {}", e);
            eprintln!("{}", err_msg);
            err_msg
        })?;
    
    entry.set_password(key)
        .map_err(|e| {
            let err_msg = format!("Failed to save API key: {}", e);
            eprintln!("{}", err_msg);
            err_msg
        })?;
    
    println!("API key saved successfully to keychain");
    
    // Verify the save worked by immediately reading it back
    match entry.get_password() {
        Ok(saved_key) => {
            if saved_key == key {
                println!("Verified: API key read back successfully from keychain");
                
                // Update cache with new key
                if let Ok(mut cache) = API_KEY_CACHE.lock() {
                    *cache = Some(key.to_string());
                    println!("✓ Cache updated with new API key");
                }
                
                Ok(())
            } else {
                let err_msg = "API key was saved but verification failed (key mismatch)".to_string();
                eprintln!("{}", err_msg);
                Err(err_msg)
            }
        },
        Err(e) => {
            let err_msg = format!("API key was saved but cannot be read back: {}", e);
            eprintln!("{}", err_msg);
            Err(err_msg)
        }
    }
}

pub fn get_api_key() -> Result<Option<String>, String> {
    // Check cache first
    if let Ok(cache) = API_KEY_CACHE.lock() {
        if cache.is_some() {
            println!("✓ API key retrieved from cache (no keychain access needed)");
            return Ok(cache.clone());
        }
    }
    
    println!("Attempting to get API key from keychain (service: {}, account: {})", SERVICE_NAME, ACCOUNT_NAME);
    
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| {
            let err_msg = format!("Failed to create keyring entry for reading: {}", e);
            eprintln!("{}", err_msg);
            err_msg
        })?;
    
    match entry.get_password() {
        Ok(key) => {
            println!("✓ API key retrieved from keychain successfully (length: {})", key.len());
            
            // Update cache for future calls
            if let Ok(mut cache) = API_KEY_CACHE.lock() {
                *cache = Some(key.clone());
                println!("✓ API key cached in memory");
            }
            
            Ok(Some(key))
        },
        Err(keyring::Error::NoEntry) => {
            println!("✗ No API key found in keychain (NoEntry error)");
            Ok(None)
        },
        Err(e) => {
            let err_msg = format!("Failed to get API key: {:?}", e);
            eprintln!("✗ Error retrieving API key: {}", err_msg);
            Err(err_msg)
        },
    }
}

pub fn get_api_key_masked() -> Result<Option<String>, String> {
    match get_api_key()? {
        Some(key) => {
            if key.len() <= 4 {
                Ok(Some("••••".to_string()))
            } else {
                let last_four = &key[key.len() - 4..];
                Ok(Some(format!("••••••••{}", last_four)))
            }
        }
        None => Ok(None),
    }
}

pub fn delete_api_key() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| format!("Failed to create keyring entry: {}", e))?;
    
    match entry.delete_credential() {
        Ok(_) => {
            // Clear cache
            if let Ok(mut cache) = API_KEY_CACHE.lock() {
                *cache = None;
                println!("✓ API key cache cleared");
            }
            Ok(())
        },
        Err(keyring::Error::NoEntry) => {
            // Clear cache anyway
            if let Ok(mut cache) = API_KEY_CACHE.lock() {
                *cache = None;
            }
            Ok(()) // Already deleted
        },
        Err(e) => Err(format!("Failed to delete API key: {}", e)),
    }
}
