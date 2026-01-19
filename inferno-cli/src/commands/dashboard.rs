use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::instance::InstanceManager;

#[derive(Args)]
pub struct DashboardArgs {
    /// Port to run dashboard on (default 3000, avoid 8080 which is stealth P2P)
    #[arg(short, long, default_value = "3000")]
    pub port: u16,

    /// Bind address
    #[arg(short, long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Open browser automatically
    #[arg(short, long)]
    pub open: bool,
}

#[derive(Clone)]
pub struct AppState {
    pub instance_manager: Arc<RwLock<InstanceManager>>,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.to_string()),
        }
    }
}

#[derive(Serialize)]
struct DashboardStats {
    total_instances: usize,
    running_instances: usize,
    total_peers: u64,
    total_blocks: u64,
    uptime: String,
}

#[derive(Serialize)]
struct InstanceInfo {
    instance_id: u32,
    status: String,
    network: String,
    p2p_port: u16,
    rpc_port: u16,
    peers: Option<u64>,
    block_height: Option<u64>,
    mining: Option<bool>,
    uptime: Option<String>,
}

#[derive(Deserialize)]
struct InstanceAction {
    instance_id: u32,
    action: String,
}

pub async fn run(args: DashboardArgs) -> Result<()> {
    let bind_addr = format!("{}:{}", args.bind, args.port);
    let url = format!("http://{}:{}", args.bind, args.port);

    println!(
        "  {} Starting dashboard server...",
        "🌐".bright_blue()
    );

    let instance_manager = InstanceManager::new()?;
    let app_state = AppState {
        instance_manager: Arc::new(RwLock::new(instance_manager)),
    };

    println!(
        "  {} Dashboard available at: {}",
        "✓".bright_green(),
        url.bright_cyan().underline()
    );
    println!();
    println!(
        "  {} Press {} to stop",
        "ℹ".dimmed(),
        "Ctrl+C".bright_yellow()
    );

    if args.open {
        let _ = open_browser(&url);
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .route("/api/stats", web::get().to(get_stats))
            .route("/api/instances", web::get().to(get_instances))
            .route("/api/instances/{id}", web::get().to(get_instance))
            .route("/api/instances/{id}/action", web::post().to(instance_action))
            .route("/api/instances/{id}/logs", web::get().to(get_instance_logs))
            .route("/api/peers", web::get().to(get_peers))
            .route("/api/blocks", web::get().to(get_blocks))
            .route("/api/config/{id}", web::get().to(get_config))
            .route("/api/config/{id}", web::put().to(update_config))
            .route("/api/docker/containers", web::get().to(get_docker_containers))
            .route("/api/remotes", web::get().to(get_remotes))
            .route("/api/health", web::get().to(health_check))
            .service(Files::new("/", get_static_dir()).index_file("index.html"))
    })
    .bind(&bind_addr)
    .context(format!("Failed to bind to {}", bind_addr))?
    .run()
    .await
    .context("Server error")?;

    Ok(())
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn get_stats(data: web::Data<AppState>) -> impl Responder {
    let manager = data.instance_manager.read().await;

    let instances = match manager.list_instances() {
        Ok(i) => i,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&e.to_string()));
        }
    };

    let total_instances = instances.len();
    let running_instances = instances
        .iter()
        .filter(|i| matches!(i.status, crate::instance::InstanceStatus::Running))
        .count();

    let stats = DashboardStats {
        total_instances,
        running_instances,
        total_peers: 0,
        total_blocks: 0,
        uptime: "0d 0h 0m".to_string(),
    };

    HttpResponse::Ok().json(ApiResponse::success(stats))
}

async fn get_instances(data: web::Data<AppState>) -> impl Responder {
    let manager = data.instance_manager.read().await;

    let instances = match manager.list_instances() {
        Ok(i) => i,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&e.to_string()));
        }
    };

    let instance_infos: Vec<InstanceInfo> = instances
        .into_iter()
        .map(|i| {
            let (network, p2p_port, rpc_port) = if let Some(ref config) = i.config {
                (
                    config.network.network.clone(),
                    config.network.p2p_port,
                    config.network.rpc_port,
                )
            } else {
                ("unknown".to_string(), 0, 0)
            };

            InstanceInfo {
                instance_id: i.instance_id,
                status: format!("{:?}", i.status),
                network,
                p2p_port,
                rpc_port,
                peers: None,
                block_height: None,
                mining: None,
                uptime: i.uptime,
            }
        })
        .collect();

    HttpResponse::Ok().json(ApiResponse::success(instance_infos))
}

async fn get_instance(
    data: web::Data<AppState>,
    path: web::Path<u32>,
) -> impl Responder {
    let instance_id = path.into_inner();
    let manager = data.instance_manager.read().await;

    if !manager.instance_exists(instance_id) {
        return HttpResponse::NotFound()
            .json(ApiResponse::<()>::error("Instance not found"));
    }

    let config = match manager.load_config(instance_id) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&e.to_string()));
        }
    };

    let pid = manager.get_pid(instance_id).ok().flatten();
    let is_running = pid.map(|p| manager.is_process_running(p)).unwrap_or(false);

    let info = InstanceInfo {
        instance_id,
        status: if is_running { "Running" } else { "Stopped" }.to_string(),
        network: config.network.network,
        p2p_port: config.network.p2p_port,
        rpc_port: config.network.rpc_port,
        peers: None,
        block_height: None,
        mining: Some(config.mining.enabled),
        uptime: None,
    };

    HttpResponse::Ok().json(ApiResponse::success(info))
}

async fn instance_action(
    data: web::Data<AppState>,
    path: web::Path<u32>,
    body: web::Json<InstanceAction>,
) -> impl Responder {
    let instance_id = path.into_inner();
    let action = &body.action;

    match action.as_str() {
        "start" => {
            match crate::commands::node::start(instance_id, false, false, "devnet").await {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::success("started")),
                Err(e) => {
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&e.to_string()))
                }
            }
        }
        "stop" => {
            match crate::commands::node::stop(instance_id, false, false).await {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::success("stopped")),
                Err(e) => {
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&e.to_string()))
                }
            }
        }
        "restart" => {
            match crate::commands::node::restart(instance_id, false).await {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::success("restarted")),
                Err(e) => {
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&e.to_string()))
                }
            }
        }
        _ => HttpResponse::BadRequest().json(ApiResponse::<()>::error("Unknown action")),
    }
}

async fn get_instance_logs(
    data: web::Data<AppState>,
    path: web::Path<u32>,
    query: web::Query<LogsQuery>,
) -> impl Responder {
    let instance_id = path.into_inner();
    let manager = data.instance_manager.read().await;

    if !manager.instance_exists(instance_id) {
        return HttpResponse::NotFound()
            .json(ApiResponse::<()>::error("Instance not found"));
    }

    let config = match manager.load_config(instance_id) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&e.to_string()));
        }
    };

    let log_file = config.node.data_dir.join("node.log");
    if !log_file.exists() {
        return HttpResponse::Ok().json(ApiResponse::success(Vec::<String>::new()));
    }

    let content = match std::fs::read_to_string(&log_file) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&e.to_string()));
        }
    };

    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let tail = query.tail.unwrap_or(100) as usize;
    let start = lines.len().saturating_sub(tail);
    let result: Vec<String> = lines[start..].to_vec();

    HttpResponse::Ok().json(ApiResponse::success(result))
}

#[derive(Deserialize)]
struct LogsQuery {
    tail: Option<u32>,
}

async fn get_peers(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(Vec::<String>::new()))
}

async fn get_blocks(data: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(Vec::<String>::new()))
}

async fn get_config(
    data: web::Data<AppState>,
    path: web::Path<u32>,
) -> impl Responder {
    let instance_id = path.into_inner();
    let manager = data.instance_manager.read().await;

    match manager.load_config(instance_id) {
        Ok(config) => HttpResponse::Ok().json(ApiResponse::success(config)),
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&e.to_string()))
        }
    }
}

async fn update_config(
    data: web::Data<AppState>,
    path: web::Path<u32>,
    body: web::Json<crate::config::InfernoConfig>,
) -> impl Responder {
    let instance_id = path.into_inner();
    let manager = data.instance_manager.read().await;

    match manager.save_config(instance_id, &body) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success("updated")),
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&e.to_string()))
        }
    }
}

async fn get_docker_containers() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(Vec::<String>::new()))
}

async fn get_remotes() -> impl Responder {
    match crate::commands::remote::RemotesFile::load() {
        Ok(remotes) => HttpResponse::Ok().json(ApiResponse::success(remotes.remotes)),
        Err(e) => {
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error(&e.to_string()))
        }
    }
}

fn get_static_dir() -> String {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let static_dir = dir.join("dashboard");
            if static_dir.exists() {
                return static_dir.to_string_lossy().to_string();
            }
        }
    }

    let fallback = dirs::home_dir()
        .map(|h| h.join(".inferno/dashboard"))
        .unwrap_or_else(|| std::path::PathBuf::from("./dashboard"));

    if !fallback.exists() {
        let _ = std::fs::create_dir_all(&fallback);
        let _ = create_default_dashboard(&fallback);
    }

    fallback.to_string_lossy().to_string()
}

fn create_default_dashboard(dir: &std::path::Path) -> Result<()> {
    let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Inferno Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://unpkg.com/lucide@latest"></script>
    <style>
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        .animate-pulse-slow { animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite; }
    </style>
</head>
<body class="bg-gray-900 text-white min-h-screen">
    <div id="app" class="container mx-auto px-4 py-8">
        <!-- Header -->
        <header class="mb-8">
            <div class="flex items-center gap-4">
                <div class="text-4xl">🔥</div>
                <div>
                    <h1 class="text-3xl font-bold bg-gradient-to-r from-orange-500 to-red-500 bg-clip-text text-transparent">
                        Inferno Dashboard
                    </h1>
                    <p class="text-gray-400">Node Management Console</p>
                </div>
            </div>
        </header>

        <!-- Stats Cards -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8" id="stats">
            <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <div class="text-gray-400 text-sm mb-1">Total Instances</div>
                <div class="text-3xl font-bold" id="total-instances">-</div>
            </div>
            <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <div class="text-gray-400 text-sm mb-1">Running</div>
                <div class="text-3xl font-bold text-green-500" id="running-instances">-</div>
            </div>
            <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <div class="text-gray-400 text-sm mb-1">Connected Peers</div>
                <div class="text-3xl font-bold text-blue-500" id="total-peers">-</div>
            </div>
            <div class="bg-gray-800 rounded-lg p-6 border border-gray-700">
                <div class="text-gray-400 text-sm mb-1">Block Height</div>
                <div class="text-3xl font-bold text-purple-500" id="block-height">-</div>
            </div>
        </div>

        <!-- Instances -->
        <div class="bg-gray-800 rounded-lg border border-gray-700 mb-8">
            <div class="p-4 border-b border-gray-700 flex justify-between items-center">
                <h2 class="text-xl font-semibold">Instances</h2>
                <button onclick="refreshInstances()" class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm">
                    Refresh
                </button>
            </div>
            <div id="instances" class="p-4">
                <div class="text-gray-400 animate-pulse-slow">Loading...</div>
            </div>
        </div>

        <!-- Logs -->
        <div class="bg-gray-800 rounded-lg border border-gray-700">
            <div class="p-4 border-b border-gray-700 flex justify-between items-center">
                <h2 class="text-xl font-semibold">Live Logs</h2>
                <select id="log-instance" onchange="refreshLogs()" class="bg-gray-700 rounded-lg px-3 py-2 text-sm">
                    <option value="1">Instance 1</option>
                </select>
            </div>
            <div id="logs" class="p-4 font-mono text-sm bg-gray-900 rounded-b-lg max-h-96 overflow-auto">
                <div class="text-gray-400">No logs available</div>
            </div>
        </div>
    </div>

    <script>
        const API = '';
        
        async function fetchStats() {
            try {
                const res = await fetch(`${API}/api/stats`);
                const data = await res.json();
                if (data.success) {
                    document.getElementById('total-instances').textContent = data.data.total_instances;
                    document.getElementById('running-instances').textContent = data.data.running_instances;
                    document.getElementById('total-peers').textContent = data.data.total_peers;
                    document.getElementById('block-height').textContent = data.data.total_blocks;
                }
            } catch (e) {
                console.error('Failed to fetch stats:', e);
            }
        }

        async function refreshInstances() {
            try {
                const res = await fetch(`${API}/api/instances`);
                const data = await res.json();
                const container = document.getElementById('instances');
                
                if (!data.success || data.data.length === 0) {
                    container.innerHTML = '<div class="text-gray-400">No instances configured</div>';
                    return;
                }

                container.innerHTML = data.data.map(inst => `
                    <div class="flex items-center justify-between p-4 bg-gray-700/50 rounded-lg mb-2">
                        <div class="flex items-center gap-4">
                            <div class="w-3 h-3 rounded-full ${inst.status === 'Running' ? 'bg-green-500' : 'bg-red-500'}"></div>
                            <div>
                                <div class="font-semibold">Instance ${inst.instance_id}</div>
                                <div class="text-sm text-gray-400">${inst.network} • P2P: ${inst.p2p_port} • RPC: ${inst.rpc_port}</div>
                            </div>
                        </div>
                        <div class="flex gap-2">
                            ${inst.status === 'Running' 
                                ? `<button onclick="instanceAction(${inst.instance_id}, 'stop')" class="px-3 py-1 bg-red-600 hover:bg-red-700 rounded text-sm">Stop</button>`
                                : `<button onclick="instanceAction(${inst.instance_id}, 'start')" class="px-3 py-1 bg-green-600 hover:bg-green-700 rounded text-sm">Start</button>`
                            }
                            <button onclick="instanceAction(${inst.instance_id}, 'restart')" class="px-3 py-1 bg-yellow-600 hover:bg-yellow-700 rounded text-sm">Restart</button>
                        </div>
                    </div>
                `).join('');

                // Update log instance selector
                const select = document.getElementById('log-instance');
                select.innerHTML = data.data.map(inst => 
                    `<option value="${inst.instance_id}">Instance ${inst.instance_id}</option>`
                ).join('');
            } catch (e) {
                console.error('Failed to fetch instances:', e);
            }
        }

        async function instanceAction(id, action) {
            try {
                await fetch(`${API}/api/instances/${id}/action`, {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ instance_id: id, action })
                });
                setTimeout(refreshInstances, 1000);
            } catch (e) {
                console.error('Action failed:', e);
            }
        }

        async function refreshLogs() {
            const instanceId = document.getElementById('log-instance').value;
            try {
                const res = await fetch(`${API}/api/instances/${instanceId}/logs?tail=100`);
                const data = await res.json();
                const container = document.getElementById('logs');
                
                if (!data.success || data.data.length === 0) {
                    container.innerHTML = '<div class="text-gray-400">No logs available</div>';
                    return;
                }

                container.innerHTML = data.data.map(line => {
                    let color = 'text-gray-300';
                    if (line.includes('ERROR') || line.includes('error')) color = 'text-red-400';
                    else if (line.includes('WARN') || line.includes('warn')) color = 'text-yellow-400';
                    else if (line.includes('INFO') || line.includes('info')) color = 'text-green-400';
                    else if (line.includes('DEBUG') || line.includes('debug')) color = 'text-blue-400';
                    return `<div class="${color}">${escapeHtml(line)}</div>`;
                }).join('');
                
                container.scrollTop = container.scrollHeight;
            } catch (e) {
                console.error('Failed to fetch logs:', e);
            }
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Initial load
        fetchStats();
        refreshInstances();
        refreshLogs();

        // Auto-refresh
        setInterval(fetchStats, 5000);
        setInterval(refreshInstances, 10000);
        setInterval(refreshLogs, 3000);
    </script>
</body>
</html>"#;

    std::fs::write(dir.join("index.html"), index_html)?;
    Ok(())
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }

    Ok(())
}
