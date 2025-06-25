use rusqlite::{Connection, Result, params};
use crate::error::LCoreError;

/// Database path within the Cartesi machine
const DB_PATH: &str = "/data/iot.db";

/// Database connection wrapper
pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Initialize the database with IoT schema
    pub fn new() -> Result<Self, LCoreError> {
        let conn = Connection::open(DB_PATH)?;
        
        // Create the database schema
        let schema = include_str!("../db/schema.sql");
        conn.execute_batch(schema)?;
        
        println!("lcore-node: Database initialized at {}", DB_PATH);
        
        Ok(Database { conn })
    }
    
    /// Insert a new device into the devices table
    pub fn insert_device(&self, device_id: &str, did_document: &str, public_key: &str) -> Result<(), LCoreError> {
        // Insert or ignore device row
        self.conn.execute(
            "INSERT OR IGNORE INTO devices (id, did_document, public_key) VALUES (?1, ?2, ?3)",
            params![device_id, did_document, public_key],
        )?;

        // Ensure there is a counter row initialised to 0
        self.conn.execute(
            "INSERT OR IGNORE INTO device_counters (device_id, counter) VALUES (?1, 0)",
            params![device_id],
        )?;

        Ok(())
    }
    
    /// Insert encrypted sensor data
    pub fn insert_sensor_data(
        &self,
        device_id: &str,
        encrypted_payload: &[u8],
        stage1_key_hash: &str,
        stage2_key_hash: &str,
        counter: u64,
        timestamp: &str,
    ) -> Result<(), LCoreError> {
        self.conn.execute(
            "INSERT INTO sensor_data (device_id, encrypted_payload, stage1_key_hash, stage2_key_hash, counter, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![device_id, encrypted_payload, stage1_key_hash, stage2_key_hash, counter, timestamp],
        )?;
        Ok(())
    }
    
    /// Atomically increment and fetch the next message counter for a device.
    pub fn next_message_counter(&self, device_id: &str) -> Result<u64, LCoreError> {
        // Upsert and increment atomically via ON CONFLICT
        self.conn.execute(
            "INSERT INTO device_counters (device_id, counter) VALUES (?1, 1)
             ON CONFLICT(device_id) DO UPDATE SET counter = counter + 1",
            params![device_id],
        )?;

        let counter: u64 = self.conn.query_row(
            "SELECT counter FROM device_counters WHERE device_id = ?1",
            params![device_id],
            |row| row.get(0),
        )?;
        Ok(counter)
    }
    
    /// Get the latest sensor data for a device
    pub fn get_latest_sensor_data(&self, device_id: &str) -> Result<Option<SensorDataRow>, LCoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, device_id, encrypted_payload, stage1_key_hash, stage2_key_hash, counter, timestamp 
             FROM sensor_data 
             WHERE device_id = ?1 
             ORDER BY timestamp DESC 
             LIMIT 1"
        )?;
        
        let mut rows = stmt.query(params![device_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(SensorDataRow {
                id: row.get(0)?,
                device_id: row.get(1)?,
                encrypted_payload: row.get(2)?,
                stage1_key_hash: row.get(3)?,
                stage2_key_hash: row.get(4)?,
                counter: row.get(5)?,
                timestamp: row.get(6)?,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Insert analytics data
    pub fn insert_analytics(
        &self,
        device_id: &str,
        metric_type: &str,
        value: f64,
        time_window: &str,
    ) -> Result<(), LCoreError> {
        self.conn.execute(
            "INSERT INTO analytics (device_id, metric_type, value, time_window) VALUES (?1, ?2, ?3, ?4)",
            params![device_id, metric_type, value, time_window],
        )?;
        Ok(())
    }
    
    /// Get analytics for a device
    pub fn get_analytics(&self, device_id: &str, metric_type: &str) -> Result<Vec<AnalyticsRow>, LCoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, device_id, metric_type, value, time_window, calculated_at 
             FROM analytics 
             WHERE device_id = ?1 AND metric_type = ?2 
             ORDER BY calculated_at DESC"
        )?;
        
        let rows = stmt.query_map(params![device_id, metric_type], |row| {
            Ok(AnalyticsRow {
                id: row.get(0)?,
                device_id: row.get(1)?,
                metric_type: row.get(2)?,
                value: row.get(3)?,
                time_window: row.get(4)?,
                calculated_at: row.get(5)?,
            })
        })?;
        
        let mut analytics = Vec::new();
        for row in rows {
            analytics.push(row?);
        }
        
        Ok(analytics)
    }

    /// Fetch stored public key JSON for a device, if present
    pub fn get_device_public_key(&self, device_id: &str) -> Result<Option<String>, LCoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT public_key FROM devices WHERE id = ?1 LIMIT 1",
        )?;
        let mut rows = stmt.query(params![device_id])?;
        if let Some(row) = rows.next()? {
            let key: String = row.get(0)?;
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }
}

/// Sensor data row structure
#[derive(Debug, Clone)]
pub struct SensorDataRow {
    pub id: i32,
    pub device_id: String,
    pub encrypted_payload: Vec<u8>,
    pub stage1_key_hash: String,
    pub stage2_key_hash: String,
    pub counter: u64,
    pub timestamp: String,
}

/// Analytics row structure
#[derive(Debug, Clone)]
pub struct AnalyticsRow {
    pub id: i32,
    pub device_id: String,
    pub metric_type: String,
    pub value: f64,
    pub time_window: String,
    pub calculated_at: String,
} 