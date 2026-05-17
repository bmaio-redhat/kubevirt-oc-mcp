mod config;
mod mcp;
mod tools;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, warn};

use config::Config;
use mcp::protocol::{Capabilities, InitializeResult, Request, Response, ServerInfo, ToolCallResult, all_tools};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_env("OC_MCP_LOG")
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cfg = Config::from_env();
    info!("kubevirt-oc-mcp starting. oc: {}, virtctl: {}", cfg.oc_path, cfg.virtctl_path);

    run_server(cfg).await;
}

async fn run_server(cfg: Config) {
    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    info!("MCP stdio server ready.");

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = Response::err(None, -32700, format!("Parse error: {}", e));
                send_response(&mut stdout, &resp).await;
                continue;
            }
        };

        if request.id.is_none() {
            continue;
        }

        let response = handle_request(request, &cfg).await;
        send_response(&mut stdout, &response).await;
    }

    info!("stdin closed, shutting down.");
}

async fn handle_request(req: Request, cfg: &Config) -> Response {
    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".into(),
                capabilities: Capabilities { tools: json!({}) },
                server_info: ServerInfo {
                    name: "kubevirt-oc-mcp".into(),
                    version: env!("CARGO_PKG_VERSION").into(),
                },
            };
            Response::ok(id, serde_json::to_value(result).unwrap())
        }

        "tools/list" => Response::ok(id, json!({ "tools": all_tools() })),

        "tools/call" => {
            let params = req.params.unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let tool_params =
                params.get("arguments").cloned().unwrap_or(Value::Object(Default::default()));

            let result = dispatch_tool(tool_name, &tool_params, cfg).await;
            Response::ok(id, serde_json::to_value(result).unwrap())
        }

        "ping" => Response::ok(id, json!({})),

        method => {
            warn!("Unknown method: {}", method);
            Response::err(id, -32601, format!("Method not found: {}", method))
        }
    }
}

async fn dispatch_tool(name: &str, params: &Value, cfg: &Config) -> ToolCallResult {
    match name {
        "oc_get" => tools::oc::oc_get(params, cfg).await,
        "oc_apply_yaml" => tools::oc::oc_apply_yaml(params, cfg).await,
        "oc_delete" => tools::oc::oc_delete(params, cfg).await,
        "oc_wait" => tools::oc::oc_wait(params, cfg).await,
        "oc_logs" => tools::oc::oc_logs(params, cfg).await,
        "oc_exec" => tools::oc::oc_exec(params, cfg).await,
        "virtctl_migrate" => tools::virtctl::virtctl_migrate(params, cfg).await,
        "virtctl_pause" => tools::virtctl::virtctl_pause(params, cfg).await,
        "virtctl_unpause" => tools::virtctl::virtctl_unpause(params, cfg).await,
        "virtctl_ssh" => tools::virtctl::virtctl_ssh(params, cfg).await,
        "cleanup_namespace" => tools::oc::cleanup_namespace(params, cfg).await,
        unknown => ToolCallResult::error(format!(
            "Unknown tool: '{}'. Available: oc_get, oc_apply_yaml, oc_delete, oc_wait, oc_logs, oc_exec, virtctl_migrate, virtctl_pause, virtctl_unpause, virtctl_ssh, cleanup_namespace",
            unknown
        )),
    }
}

async fn send_response(stdout: &mut tokio::io::Stdout, response: &Response) {
    match serde_json::to_string(response) {
        Ok(json) => {
            let line = format!("{}\n", json);
            if let Err(e) = stdout.write_all(line.as_bytes()).await {
                error!("Failed to write response: {}", e);
            }
            if let Err(e) = stdout.flush().await {
                error!("Failed to flush stdout: {}", e);
            }
        }
        Err(e) => error!("Failed to serialize response: {}", e),
    }
}
