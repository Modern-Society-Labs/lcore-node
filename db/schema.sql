-- IoT-L{CORE} Database Schema
-- Phase 3: IoT Application Logic in Cartesi VM

-- Device registry to store authenticated device information
CREATE TABLE IF NOT EXISTS devices (
  id TEXT PRIMARY KEY,
  did_document TEXT NOT NULL,
  public_key TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Per-device message counter for deterministic nonce generation
CREATE TABLE IF NOT EXISTS device_counters (
  device_id TEXT PRIMARY KEY,
  counter   INTEGER NOT NULL
);

-- Table to store encrypted IoT sensor data payloads from devices
CREATE TABLE IF NOT EXISTS sensor_data (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT NOT NULL,
  encrypted_payload BLOB NOT NULL,
  stage1_key_hash TEXT NOT NULL,
  stage2_key_hash TEXT NOT NULL,
  counter INTEGER NOT NULL,
  timestamp TIMESTAMP NOT NULL,
  FOREIGN KEY (device_id) REFERENCES devices(id)
);

-- Table to cache results of analytics and other computations
CREATE TABLE IF NOT EXISTS analytics (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_id TEXT NOT NULL,
  metric_type TEXT NOT NULL,
  value REAL NOT NULL,
  time_window TEXT NOT NULL,
  calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
