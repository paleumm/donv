use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::Path,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use nvml_wrapper::Nvml;
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::time::{interval, Duration};
use tower_http::services::ServeDir;
use tracing::info;

static NVML_INSTANCE: Lazy<Nvml> = Lazy::new(|| Nvml::init().expect("Failed to initialize NVML"));

#[derive(Serialize)]
struct GpuInfo {
    memory_total_mib: u64,
    memory_used_mib: u64,
    memory_util: u32,
    power_usage: u32,
    id: u32,
    utilization_gpu: u32,
    cuda_version: i32,
    name: String,
    driver_version: String,
    nvml_version: String,
}

async fn list_gpu() -> Json<Vec<GpuInfo>> {
    let nvml = &*NVML_INSTANCE; // safe, already initialized

    let count = nvml.device_count().unwrap();
    let mut gpus = Vec::new();

    let driver_version = nvml.sys_driver_version().unwrap();
    let cuda_version = nvml.sys_cuda_driver_version().unwrap();
    let nvml_version = nvml.sys_nvml_version().unwrap();

    for i in 0..count {
        if let Ok(device) = nvml.device_by_index(i) {
            let mem = device.memory_info().unwrap();
            let util = device.utilization_rates().unwrap();
            let mem_util = util.memory;
            let pow = device.power_usage().unwrap(); // milliwatts

            gpus.push(GpuInfo {
                id: i,
                memory_total_mib: bytes_to_mib(mem.total),
                memory_used_mib: bytes_to_mib(mem.used),
                utilization_gpu: util.gpu,
                power_usage: pow / 1000,
                memory_util: mem_util,
                cuda_version: cuda_version.clone(),
                driver_version: driver_version.clone(),
                nvml_version: nvml_version.clone(),
                name: device.name().unwrap(),
            });
        }
    }

    Json(gpus)
}

async fn gpu_by_id(Path(id): Path<u32>) -> Result<Json<GpuInfo>, (axum::http::StatusCode, String)> {
    tracing::info!("Handling request for GPU {}", id);
    let nvml = &*NVML_INSTANCE;
    let count = nvml.device_count().unwrap();
    if id > count {
        return Err((
            axum::http::StatusCode::NOT_FOUND,
            format!("GPU {} not found", id),
        ));
    }

    let driver_version = nvml.sys_driver_version().unwrap();
    let cuda_version = nvml.sys_cuda_driver_version().unwrap();
    let nvml_version = nvml.sys_nvml_version().unwrap();

    match nvml.device_by_index(id) {
        Ok(device) => {
            let mem = device.memory_info().unwrap();
            let util = device.utilization_rates().unwrap();
            let mem_util = util.memory;
            let pow = device.power_usage().unwrap(); // milliwatts

            tracing::info!(
                "GPU {}: name = {}, memory = {} MiB used / {} MiB total, power = {}W, util = {}%",
                id,
                device.name().unwrap(),
                bytes_to_mib(mem.used),
                bytes_to_mib(mem.total),
                pow / 1000,
                util.gpu
            );

            Ok(Json(GpuInfo {
                id: id,
                memory_total_mib: bytes_to_mib(mem.total),
                memory_used_mib: bytes_to_mib(mem.used),
                utilization_gpu: util.gpu,
                power_usage: pow / 1000,
                memory_util: mem_util,
                cuda_version: cuda_version.clone(),
                driver_version: driver_version.clone(),
                nvml_version: nvml_version.clone(),
                name: device.name().unwrap(),
            }))
        }
        Err(_) => {
            tracing::error!("Failed to get GPU {}", id);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get GPU {} info", id),
            ))
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut interval = interval(Duration::from_secs(1));
    loop {
        interval.tick().await;

        let Json(gpus) = list_gpu().await; // implement this function to return Vec<GpuInfo>
        let payload = serde_json::to_string(&gpus).unwrap();

        if socket.send(Message::Text(payload)).await.is_err() {
            break; // client disconnected
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/gpus", get(list_gpu))
        .route("/gpu/:id", get(gpu_by_id))
        .nest_service("/", ServeDir::new("static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:26501")
        .await
        .unwrap();

    info!("DoNV server started at http://0.0.0.0:26501");
    axum::serve(listener, app).await.unwrap();
}

fn bytes_to_mib(bytes: u64) -> u64 {
    bytes / 1024 / 1024
}
