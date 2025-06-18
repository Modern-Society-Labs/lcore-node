use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;
use tracing::info;

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = match req.uri().path() {
        "/health" => {
            let health_status = json::object! {
                status: "healthy",
                service: "lcore-node",
                version: "0.1.0"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(health_status.dump()))
                .unwrap()
        }
        "/inspect" => {
            // Cartesi inspect endpoint - returns application state
            let inspect_data = json::object! {
                message: "lcore-node Cartesi application",
                status: "running",
                capabilities: ["iot_data_processing", "dual_encryption"]
            };
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(inspect_data.dump()))
                .unwrap()
        }
        "/advance" => {
            // Cartesi advance endpoint - processes inputs
            let advance_response = json::object! {
                status: "accept",
                message: "Input processed successfully"
            };
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(advance_response.dump()))
                .unwrap()
        }
        _ => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap()
        }
    };

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    tracing_subscriber::fmt::init();

    info!("Starting lcore-node Cartesi application...");

    // Create service
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    // Bind to the address
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let server = Server::bind(&addr).serve(make_svc);

    info!("lcore-node server running on http://{}", addr);

    // Run the server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }

    Ok(())
} 