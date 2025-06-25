// lcore-node: Cartesi-based IoT data processing node
// This module provides the core functionality for processing IoT data
// within a Cartesi rollups environment

pub mod error;
pub mod database;
pub mod encryption;
pub mod device_auth;

use serde::{Deserialize, Serialize};

// Basic IoT data structure following the documentation schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTData {
    pub device_id: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

// Device registration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegistration {
    pub did: String,
    pub did_document: String,
    pub signature: String,
}

// Encrypted sensor data payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedSensorPayload {
    pub device_id: String,
    pub encrypted_data: Vec<u8>,
    pub stage1_key_hash: String,
    pub stage2_key_hash: String,
    pub timestamp: String,
}

// Application state for Cartesi integration
#[derive(Debug, Clone)]
pub struct AppState {
    // Database is now handled by the Database struct
    // Encryption is handled by Stage1Encryption and Stage2Encryption
    // Device authentication is handled by device_auth module
} 