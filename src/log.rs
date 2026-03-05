use crate::config::AppConfig;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use std::path::Path;
use std::sync::Once;
use std::time::Instant;
use tracing::{error, info, warn};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, fmt};
use uuid::Uuid;

/// 初始化日志系统
///
/// 1. 控制台日志 (stdout): 使用紧凑格式，方便 Docker logs 查看，不包含过多干扰信息。
/// 2. 文件日志 (file): 使用 JSON 格式，包含完整结构化信息，方便后续分析。
pub fn init_logging(config: &AppConfig) {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        println!("Initializing logging...");

        // 解析日志级别
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log.level));

        // --- Layer 1: 控制台输出 (Docker logs) ---
        // 使用 Compact 格式，去除时间戳（Docker 自身会打时间戳），保持整洁
        let stdout_layer = fmt::layer()
            .compact()
            .with_target(false) // 隐藏模块路径，只显示消息
            .with_file(false)
            .with_level(true)
            .with_ansi(true) // 支持颜色
            .with_filter(env_filter.clone());

        // --- Layer 2: 文件输出 ---
        // 解析文件路径
        let path_str = &config.log.log_file_path;
        let path = Path::new(path_str);

        // 提取目录和文件名
        let directory = path.parent().unwrap_or_else(|| Path::new("./logs"));
        let filename = path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("sub.log"));

        // 设置文件追加器 (每天轮转)
        let file_appender = tracing_appender::rolling::daily(directory, filename);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        // 必须泄漏 guard 以便在主线程结束后（或整个生命周期内）保持日志写入器开启
        std::mem::forget(_guard);

        // 文件日志使用 JSON 格式，包含所有字段
        let file_layer = fmt::layer()
            .json()
            .with_writer(non_blocking)
            .with_span_events(FmtSpan::CLOSE) // 记录请求结束时间
            .with_filter(env_filter);

        // 注册所有 Layer
        // 使用 try_init 避免重复初始化 panic
        if let Err(e) = tracing_subscriber::registry()
            .with(stdout_layer)
            .with(file_layer)
            .try_init()
        {
            eprintln!("Failed to initialize tracing subscriber: {}", e);
        }
    });
}

/// Middleware: 结构化 HTTP 请求追踪
///
/// 生成 x-request-id 并记录请求耗时。
pub async fn trace_requests(
    req: Request,
    next: Next,
) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let http_method = req.method().to_string();
    let http_path = req.uri().path().to_string();
    let headers = req.headers();

    let client_ip = extract_client_ip(headers);

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("-")
        .to_string();

    // 创建 Span，包含请求上下文
    let span = tracing::info_span!(
        "http_req",
        id = %request_id,
        method = %http_method,
        path = %http_path,
        ip = %client_ip
    );

    let _enter = span.enter();
    let start_time = Instant::now();

    let mut res = next.run(req).await;

    let duration = start_time.elapsed();
    let status_code = res.status().as_u16();

    // 注入 Request ID 到响应头
    res.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    // 根据状态码决定日志级别
    match status_code {
        500..=599 => {
            error!(
                status = status_code,
                latency_ms = duration.as_millis(),
                ua = %user_agent,
                "Internal Server Error"
            );
        }
        400..=499 => {
            warn!(
                status = status_code,
                latency_ms = duration.as_millis(),
                "Client Error"
            );
        }
        _ => {
            // 对于健康检查等高频请求，可以考虑降低级别为 DEBUG
            if http_path == "/healthz" {
                tracing::debug!(status = status_code, "health check");
            } else {
                info!(
                    status = status_code,
                    latency_ms = duration.as_millis(),
                    "Finished"
                );
            }
        }
    }

    res
}

fn extract_client_ip(headers: &HeaderMap) -> String {
    // 优先使用 X-Forwarded-For (反向代理设置)
    if let Some(xff) = headers.get("x-forwarded-for")
        && let Ok(val) = xff.to_str() {
        return val.split(',').next().unwrap_or("unknown").trim().to_string();
    }
    // 其次使用 X-Real-IP
    if let Some(xri) = headers.get("x-real-ip")
        && let Ok(val) = xri.to_str() {
        return val.to_string();
    }
    "unknown".to_string()
}
