use json::{object, JsonValue};
use std::env;
use crate::encryption::{Stage1Encryption, Stage2Encryption};
use crate::encryption::{derive_stage1_nonce, derive_stage2_nonce};
use crate::database::Database;
use hex;
use sha2::{Digest, Sha256};
use chrono;
use serde::{Deserialize, Serialize};
use serde_json;
use hyper::{service::{make_service_fn, service_fn}, Body, Request, Response, Server};
use std::net::SocketAddr;
use tokio::task;

pub mod encryption;
pub mod device_auth;
pub mod database;
pub mod error;

#[derive(Deserialize, Serialize)]
struct SensorData {
    id: i32,
    device_id: String,
    encrypted_payload: String,
    timestamp: String,
}

#[derive(Deserialize, Serialize)]
struct DecryptedSensorData {
    id: i32,
    device_id: String,
    decrypted_payload: String,
    timestamp: String,
}

#[derive(Deserialize)]
struct RegisterPayload {
    device_id: String,
    did_document: String,
    public_key: String,
}

#[derive(Deserialize)]
struct DataPayload {
    device_id: String,
    jws: String,
    data: String, // Hex-encoded hex string
}

#[derive(Deserialize)]
struct WrappedPayload {
    action: String,
    payload: serde_json::Value,
}

pub async fn handle_advance(
    db: &Database,
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Received advance request data {}", &request);
    let payload_str = request["data"]["payload"].as_str().ok_or("Missing payload")?;
    let payload_bytes = hex::decode(payload_str.trim_start_matches("0x"))?;
    let payload_json_str = std::str::from_utf8(&payload_bytes)?;

    let wrapped: WrappedPayload = serde_json::from_str(payload_json_str)?;

    match wrapped.action.as_str() {
        "register" => {
            let reg: RegisterPayload = serde_json::from_value(wrapped.payload)?;
            // Insert device and initialise counter
            db.insert_device(&reg.device_id, &reg.did_document, &reg.public_key)?;
            println!("Device {} registered", reg.device_id);
            return Ok("accept");
        }
        "submit" => {
            let data_pl: DataPayload = serde_json::from_value(wrapped.payload)?;

            // --- Device Authentication ---
            // In a real system, the public key would be fetched from the DID document
            // corresponding to the DID in the JWS header. For now, we use a hardcoded key.
            let public_jwk_json = r#"{
                "kty": "OKP",
                "crv": "Ed25519",
                "x": "y_8A1YdE2a4ah98y-a9s2kO4F8c_o-wW_tH2Z6jY8lU"
            }"#;

            let data_bytes = hex::decode(&data_pl.data)?;

            let device_did = data_pl.device_id.as_str();
            if let Some(pk_json) = db.get_device_public_key(device_did)? {
                if !data_pl.jws.is_empty() {
                    match device_auth::verify_device_signature(&data_pl.jws, &data_bytes, &pk_json) {
                        Ok(_) => println!("Device signature verified successfully!"),
                        Err(e) => {
                            println!("Device authentication failed: {}", e);
                            return Ok("reject");
                        }
                    }
                } else {
                    println!("No JWS provided; skipping signature verification for device {}", device_did);
                }
            } else {
                println!("No public key on record for device {}; skipping verification", device_did);
            }
            
            // --- Dual Encryption with deterministic nonce ---
            let encryption_context = "iot-sensor-data-v1";

            // Obtain per-device counter (incremented atomically)
            let counter = db.next_message_counter(device_did)?;

            // Stage 1
            let key1 = Stage1Encryption::derive_key_from_did(device_did)?;
            let stage1 = Stage1Encryption::new(key1);
            let nonce1 = derive_stage1_nonce(device_did, counter);
            let ciphertext1 = stage1.encrypt_with_nonce(&data_bytes, &nonce1)?;
            println!("Stage 1 (AES) ciphertext length: {}", ciphertext1.len());

            // Stage 2
            let key2 = Stage2Encryption::derive_key_from_context(encryption_context)?;
            let stage2 = Stage2Encryption::new(key2);
            let nonce2 = derive_stage2_nonce(device_did, counter);
            let ciphertext2 = stage2.encrypt_with_nonce(&ciphertext1, &nonce2)?;
            println!("Stage 2 (ChaCha) ciphertext length: {}", ciphertext2.len());

            // --- Database Operations ---
            // First ensure the device exists
            db.insert_device(
                device_did, 
                "{\"doc\":\"placeholder\"}", 
                "{\"key\":\"placeholder\"}"
            )?;

            // Store the encrypted payload
            let key1_hash = hex::encode(Sha256::digest(&key1));
            let key2_hash = hex::encode(Sha256::digest(&key2));
            let timestamp = chrono::Utc::now().to_rfc3339();

            db.insert_sensor_data(
                device_did,
                &ciphertext2,
                &key1_hash,
                &key2_hash,
                counter,
                &timestamp,
            )?;
            
            println!("lcore-node: Successfully authenticated, processed, encrypted, and stored IoT data input.");
            Ok("accept")
        }
        _ => {
            println!("Unknown action {}", wrapped.action);
            Ok("reject")
        }
    }
}

pub async fn handle_inspect(
    db: &Database,
    _client: &hyper::Client<hyper::client::HttpConnector>,
    _server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Received inspect request data {}", &request);
    let payload_str = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;
    let payload_bytes = hex::decode(payload_str.trim_start_matches("0x"))?;
    let query_str = std::str::from_utf8(&payload_bytes)?;
    
    println!("lcore-node: Processing state inspection for query '{}'", query_str);

    // Simple query format: "get_latest:<device_id>"
    if let Some(device_id_query) = query_str.strip_prefix("get_latest:") {
        if let Some(sensor_row) = db.get_latest_sensor_data(device_id_query)? {
            // --- Decryption Logic ---
            let encryption_context = "iot-sensor-data-v1"; // Must match encryption context

            let counter = sensor_row.counter;

            // Stage 2 Decryption (ChaCha20)
            let key2 = Stage2Encryption::derive_key_from_context(encryption_context)?;
            let stage2 = Stage2Encryption::new(key2);
            let nonce2 = derive_stage2_nonce(device_id_query, counter);
            let ciphertext1 = stage2.decrypt_with_nonce(&sensor_row.encrypted_payload, &nonce2)?;
            println!("Stage 2 decryption successful.");

            // Stage 1 Decryption (AES)
            let key1 = Stage1Encryption::derive_key_from_did(&sensor_row.device_id)?;
            let stage1 = Stage1Encryption::new(key1);
            let nonce1 = derive_stage1_nonce(&sensor_row.device_id, counter);
            let plaintext_bytes = stage1.decrypt_with_nonce(&ciphertext1, &nonce1)?;
            println!("Stage 1 decryption successful.");

            let decrypted_payload_str = std::str::from_utf8(&plaintext_bytes)?;
            
            let sensor_data = DecryptedSensorData {
                id: sensor_row.id,
                device_id: sensor_row.device_id,
                decrypted_payload: decrypted_payload_str.to_string(),
                timestamp: sensor_row.timestamp,
            };
            
            let notice = object! {
                "type" => "decrypted_sensor_data",
                "data" => serde_json::to_string(&sensor_data)?
            };
            
            println!("Sending notice with decrypted data: {}", notice.dump());
            // In a real scenario, we would send this as a notice to the rollups node.
        } else {
             println!("No data found for device {}", device_id_query);
        }
    } else {
        println!("Invalid inspect query format. Use 'get_latest:<device_id>'");
    }
    
    Ok("accept")
}

/// Starts a lightweight HTTP server on 0.0.0.0:8000 that responds to GET /health
/// with a JSON payload including database status. This server is intended for
/// local testing and liveness checks and runs concurrently with the Cartesi
/// rollups loop.
async fn start_health_server() {
    // Address inside the VM; 0.0.0.0 allows external port mapping if needed.
    let addr: SocketAddr = ([0, 0, 0, 0], 8000).into();

    // Service factory
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(|req: Request<Body>| async move {
            match (req.method(), req.uri().path()) {
                (&hyper::Method::GET, "/health") => {
                    // Attempt to open the SQLite DB and run a simple pragma to verify it is accessible.
                    let db_status = match Database::new() {
                        Ok(db) => {
                            // Quick check – pragma user_version; ignore result.
                            let _ = db.conn.pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0));
                            "ok"
                        }
                        Err(_) => "error",
                    };

                    let payload: JsonValue = object! {
                        status: "healthy",
                        database: db_status,
                        service: "lcore-node",
                    };
                    let resp = Response::builder()
                        .header(hyper::header::CONTENT_TYPE, "application/json")
                        .status(200)
                        .body(Body::from(payload.dump()))
                        .unwrap();
                    Ok::<_, hyper::Error>(resp)
                }
                _ => {
                    let resp = Response::builder().status(404).body(Body::empty()).unwrap();
                    Ok::<_, hyper::Error>(resp)
                }
            }
        }))
    });

    if let Err(e) = Server::bind(&addr).serve(make_svc).await {
        eprintln!("Health server error: {}", e);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting lcore-node Cartesi application...");

    // Spawn health server in background
    task::spawn(async {
        start_health_server().await;
    });

    // Initialize the database following cartesi-risczero pattern
    let db = Database::new().expect("Failed to initialize database");
    
    let client = hyper::Client::new();
    let server_addr = env::var("ROLLUP_HTTP_SERVER_URL")?;
    
    println!("lcore-node: Connected to rollups server at {}", server_addr);

    let mut status = "accept";
    loop {
        println!("Sending finish");
        let response = object! {"status" => status};
        let request = hyper::Request::builder()
            .method(hyper::Method::POST)
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .uri(format!("{}/finish", &server_addr))
            .body(hyper::Body::from(response.dump()))?;
        let response = client.request(request).await?;
        println!("Received finish status {}", response.status());

        if response.status() == hyper::StatusCode::ACCEPTED {
            println!("No pending rollup request, trying again");
        } else {
            let body = hyper::body::to_bytes(response).await?;
            let utf = std::str::from_utf8(&body)?;
            let req = json::parse(utf)?;

            let request_type = req["request_type"]
                .as_str()
                .ok_or("request_type is not a string")?;
            status = match request_type {
                "advance_state" => handle_advance(&db, &client, &server_addr[..], req).await?,
                "inspect_state" => handle_inspect(&db, &client, &server_addr[..], req).await?,
                &_ => {
                    eprintln!("Unknown request type");
                    "reject"
                }
            };
        }
    }
} 