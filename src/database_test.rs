#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Database, SensorDataRow};
    use crate::encryption::{Stage1Encryption, Stage2Encryption};
    use sha2::{Digest, Sha256};
    use hex;

    #[test]
    fn test_database_initialization() {
        // Test database creation following cartesi-risczero pattern
        let db = Database::new().expect("Failed to create database");
        
        // Verify database connection is working
        assert!(db.conn.is_ok());
    }

    #[test]
    fn test_device_insertion() {
        let db = Database::new().expect("Failed to create database");
        
        // Test device insertion following IoT schema
        let device_id = "did:example:123456789";
        let did_document = "{\"id\":\"did:example:123456789\",\"publicKey\":[]}";
        let public_key = "{\"kty\":\"OKP\",\"crv\":\"Ed25519\"}";
        
        let result = db.insert_device(device_id, did_document, public_key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sensor_data_insertion() {
        let db = Database::new().expect("Failed to create database");
        
        // First insert device
        let device_id = "did:example:123456789";
        db.insert_device(device_id, "{}", "{}").expect("Failed to insert device");
        
        // Test sensor data insertion following dual encryption pattern
        let test_data = b"temperature:23.5,humidity:45.2";
        let stage1_key = Stage1Encryption::derive_key_from_did(device_id).expect("Failed to derive key");
        let stage2_key = Stage2Encryption::derive_key_from_context("iot-sensor-data-v1").expect("Failed to derive key");
        
        let stage1_hash = hex::encode(Sha256::digest(&stage1_key));
        let stage2_hash = hex::encode(Sha256::digest(&stage2_key));
        let timestamp = "2024-01-01T00:00:00Z";
        
        let result = db.insert_sensor_data(
            device_id,
            test_data,
            &stage1_hash,
            &stage2_hash,
            1,
            timestamp,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_sensor_data_retrieval() {
        let db = Database::new().expect("Failed to create database");
        
        // Setup test data
        let device_id = "did:example:123456789";
        db.insert_device(device_id, "{}", "{}").expect("Failed to insert device");
        
        let test_data = b"temperature:23.5,humidity:45.2";
        let timestamp = "2024-01-01T00:00:00Z";
        
        db.insert_sensor_data(device_id, test_data, "hash1", "hash2", 1, timestamp)
            .expect("Failed to insert sensor data");
        
        // Test retrieval
        let result = db.get_latest_sensor_data(device_id).expect("Failed to query data");
        assert!(result.is_some());
        
        let sensor_data = result.unwrap();
        assert_eq!(sensor_data.device_id, device_id);
        assert_eq!(sensor_data.encrypted_payload, test_data);
        assert_eq!(sensor_data.timestamp, timestamp);
    }

    #[test]
    fn test_analytics_insertion() {
        let db = Database::new().expect("Failed to create database");
        
        // Test analytics insertion following IoT schema
        let device_id = "did:example:123456789";
        let metric_type = "temperature_avg";
        let value = 23.5;
        let time_window = "1h";
        
        let result = db.insert_analytics(device_id, metric_type, value, time_window);
        assert!(result.is_ok());
    }

    #[test]
    fn test_end_to_end_iot_flow() {
        // Test complete IoT data flow following cartesi-risczero pattern
        let db = Database::new().expect("Failed to create database");
        
        // 1. Device registration
        let device_did = "did:example:123456789";
        db.insert_device(device_did, "{}", "{}").expect("Device registration failed");
        
        // 2. Dual encryption
        let original_data = b"sensor_reading:temperature=23.5,humidity=45.2,pressure=1013.25";
        
        let key1 = Stage1Encryption::derive_key_from_did(device_did).expect("Key derivation failed");
        let stage1 = Stage1Encryption::new(key1);
        let ciphertext1 = stage1.encrypt(original_data).expect("Stage 1 encryption failed");
        
        let key2 = Stage2Encryption::derive_key_from_context("iot-sensor-data-v1").expect("Key derivation failed");
        let stage2 = Stage2Encryption::new(key2);
        let ciphertext2 = stage2.encrypt(&ciphertext1).expect("Stage 2 encryption failed");
        
        // 3. Database storage
        let key1_hash = hex::encode(Sha256::digest(&key1));
        let key2_hash = hex::encode(Sha256::digest(&key2));
        let timestamp = "2024-01-01T00:00:00Z";
        
        db.insert_sensor_data(device_did, &ciphertext2, &key1_hash, &key2_hash, 1, timestamp)
            .expect("Data storage failed");
        
        // 4. Data retrieval and decryption
        let sensor_row = db.get_latest_sensor_data(device_did)
            .expect("Data retrieval failed")
            .expect("No data found");
        
        let decrypted1 = stage2.decrypt(&sensor_row.encrypted_payload).expect("Stage 2 decryption failed");
        let decrypted_original = stage1.decrypt(&decrypted1).expect("Stage 1 decryption failed");
        
        assert_eq!(decrypted_original, original_data);
        
        // 5. Analytics
        db.insert_analytics(device_did, "temperature", 23.5, "1h").expect("Analytics insertion failed");
        
        let analytics = db.get_analytics(device_did, "temperature").expect("Analytics retrieval failed");
        assert!(!analytics.is_empty());
        assert_eq!(analytics[0].value, 23.5);
    }
} 