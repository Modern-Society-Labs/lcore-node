// lcore-node: Cartesi-based IoT data processing node
// This module provides the core functionality for processing IoT data
// within a Cartesi rollups environment

pub mod error;

use serde::{Deserialize, Serialize};

// Basic IoT data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IoTData {
    pub device_id: String,
    pub timestamp: u64,
    pub data: serde_json::Value,
}

// Application state for Cartesi integration
#[derive(Debug, Clone)]
pub struct AppState {
    // Future: Add SQLite database connection
    // Future: Add encryption state
} 