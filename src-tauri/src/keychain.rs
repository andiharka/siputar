use keyring::Entry;
use std::sync::Mutex;
use once_cell::sync::Lazy;

const SERVICE_NAME: &str = "com.disperpusip.playbackannouncer";
const ACCOUNT_NAME: &str = "elevenlabs-api-key";

// In-memory cache to prevent repeated keychain dialogs
static API_KEY_CACHE: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn save_api_key(key: &str) -> Result<(), String> {
    println!("[Keychain] === SAVE API KEY ===");
    println!("[Keychain] Platform: {}", std::env::consts::OS);
    println!("[Keychain] Service: {}", SERVICE_NAME);
    println!("[Keychain] Account: {}", ACCOUNT_NAME);
    
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| {
            let err_msg = format!("Failed to create keyring entry: {}", e);
            eprintln!("[Keychain] ERROR creating entry: {}", err_msg);
            err_msg
        })?;
    
    entry.set_password(key)
        .map_err(|e| {
            let err_msg = format!("Failed to save API key: {} (type: {:?})", e, e);
            eprintln!("[Keychain] ERROR saving: {}", err_msg);
            err_msg
        })?;
    
    println!("[Keychain] ✓ API key saved to platform keychain");
    
    // Verify the save worked by immediately reading it back with a FRESH Entry
    // Using a new Entry instance bypasses any internal caching
    let verify_entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| {
            let err_msg = format!("Failed to create verification entry: {}", e);
            eprintln!("[Keychain] ERROR: {}", err_msg);
            err_msg
        })?;
    
    match verify_entry.get_password() {
        Ok(saved_key) => {
            if saved_key == key {
                println!("[Keychain] ✓ Verification PASSED: Read back matches saved key");
                println!("[Keychain] ✓ PERSISTENT STORAGE CONFIRMED - will survive app restart");
                
                // Update cache with new key
                if let Ok(mut cache) = API_KEY_CACHE.lock() {
                    *cache = Some(key.to_string());
                    println!("[Keychain] ✓ In-memory cache updated");
                }
                
                Ok(())
            } else {
                let err_msg = "Verification FAILED: Key mismatch after save".to_string();
                eprintln!("[Keychain] ERROR: {}", err_msg);
                Err(err_msg)
            }
        },
        Err(e) => {
            let err_msg = format!("Verification FAILED: Cannot read back saved key: {}", e);
            eprintln!("[Keychain] ERROR: {}", err_msg);
            Err(err_msg)
        }
    }
}

pub fn get_api_key() -> Result<Option<String>, String> {
    // Check cache first
    if let Ok(cache) = API_KEY_CACHE.lock() {
        if cache.is_some() {
            println!("[Keychain] ✓ Retrieved from cache (no keychain access)");
            return Ok(cache.clone());
        }
    }
    
    println!("[Keychain] === GET API KEY ===");
    println!("[Keychain] Platform: {}", std::env::consts::OS);
    println!("[Keychain] Service: {}", SERVICE_NAME);
    println!("[Keychain] Account: {}", ACCOUNT_NAME);
    println!("[Keychain] Accessing platform keychain...");
    
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME)
        .map_err(|e| {
            let err_msg = format!("Failed to create keyring entry: {}", e);
            eprintln!("[Keychain] ERROR: {}", err_msg);
            err_msg
        })?;
    
    match entry.get_password() {
        Ok(key) => {
            println!("[Keychain] ✓ API key retrieved from keychain successfully");
            println!("[Keychain] ✓ Key length: {} characters", key.len());
            println!("[Keychain] ✓ This proves PERSISTENT storage is working!");
            
            // Update cache for future calls
            if let Ok(mut cache) = API_KEY_CACHE.lock() {
                *cache = Some(key.clone());
                println!("[Keychain] ✓ Cached for this session");
            }
            
            Ok(Some(key))
        },
        Err(keyring::Error::NoEntry) => {
            println!("[Keychain] ✗ No API key found in keychain");
            println!("[Keychain] ✗ Either never saved OR persistence failed");
            Ok(None)
        },
        Err(e) => {
            let err_msg = format!("Failed to get API key: {:?}", e);
            eprintln!("[Keychain] ERROR: {}", err_msg);
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
