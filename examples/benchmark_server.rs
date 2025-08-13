//! # Zerust Benchmark Server 性能测试程序
//!
//! 本示例用于测试 Zerust 框架的性能，并与 Go 的 Zinx 项目进行对比：
//! - 高并发连接处理能力
//! - 请求吞吐量（RPS - Requests Per Second）
//! - 请求延迟（Latency）
//! - 资源占用（CPU、内存）
//!
//! ✅ 运行方式：
//! ```bash
//! # 启动服务器
//! cargo run --release --example benchmark_server -- server
//!
//! # 在另一个终端运行客户端测试
//! cargo run --release --example benchmark_server -- client [连接数] [每连接请求数]
//! ```
//!
//! 例如：
//! ```bash
//! cargo run --release --example benchmark_server -- client 100 1000
//! ```
//! 将创建100个并发连接，每个连接发送1000个请求

use std::env;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Barrier, Semaphore, oneshot};
use tokio::time::sleep;
use zerust::datapack::DataPack;
use zerust::{DefaultRouter, Response, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("server") => run_server().await?,
        Some("client") => {
            let connections = args
                .get(2)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(100);
            let requests_per_conn = args
                .get(3)
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1000);
            run_client(connections, requests_per_conn).await?
        }
        _ => {
            println!(
                "用法: cargo run --release --example benchmark_server -- [server|client] [连接数] [每连接请求数]"
            );
            println!("  server          - 启动基准测试服务器");
            println!("  client [连接数] [每连接请求数] - 启动客户端测试");
        }
    }

    Ok(())
}

/// 运行基准测试服务器
async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // 创建关闭通道
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // 创建路由器并注册回显处理函数
    let router = Arc::new(DefaultRouter::new());
    let router_clone = router.clone();

    // 计数器，用于统计处理的请求数
    let request_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = request_counter.clone();

    // 注册高性能回显处理函数 - 不打印日志，直接返回
    router_clone.add_route(1, move |req| {
        counter_clone.fetch_add(1, Ordering::Relaxed);
        Response::new(req.msg_id(), req.data().to_vec())
    });

    // 启动服务器
    let server_addr = "127.0.0.1:8888";
    let server = Server::new(server_addr, router);
    println!("[Server] 基准测试服务器启动在 {}", server_addr);

    // 启动统计任务
    let stats_handle = tokio::spawn(async move {
        let mut last_count = 0;
        let mut last_time = Instant::now();

        loop {
            sleep(Duration::from_secs(1)).await;
            let current_count = request_counter.load(Ordering::Relaxed);
            let current_time = Instant::now();
            let elapsed = current_time.duration_since(last_time).as_secs_f64();

            let rps = (current_count - last_count) as f64 / elapsed;
            println!(
                "[Stats] 当前RPS: {:.2} req/s, 总请求数: {}",
                rps, current_count
            );

            last_count = current_count;
            last_time = current_time;
        }
    });

    // 启动服务器并等待Ctrl+C信号
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run(shutdown_rx).await {
            eprintln!("[Server] 运行时错误: {}", e);
        }
    });

    println!("[Server] 按 Ctrl+C 停止服务器...");
    tokio::signal::ctrl_c().await?;
    println!("[Server] 接收到停止信号，正在关闭...");

    // 发送关闭信号
    let _ = shutdown_tx.send(());

    // 等待服务器和统计任务完成
    let _ = server_handle.await;
    stats_handle.abort();

    println!("[Server] 服务器已关闭");
    Ok(())
}

/// 运行客户端基准测试
async fn run_client(
    connections: usize,
    requests_per_conn: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "[Client] 开始基准测试: {} 并发连接, 每连接 {} 请求",
        connections, requests_per_conn
    );

    // 创建信号量限制并发连接数
    let semaphore = Arc::new(Semaphore::new(connections));

    // 创建同步屏障，确保所有连接同时开始发送请求
    let barrier = Arc::new(Barrier::new(connections + 1)); // +1 for main thread

    // 统计数据
    let total_requests = connections * requests_per_conn;
    let completed_requests = Arc::new(AtomicUsize::new(0));
    let total_latency = Arc::new(AtomicUsize::new(0)); // 以微秒为单位

    // 启动客户端连接
    let mut handles = Vec::with_capacity(connections);

    let start_time = Instant::now();

    for i in 0..connections {
        let semaphore_clone = semaphore.clone();
        let barrier_clone = barrier.clone();
        let completed_clone = completed_requests.clone();
        let latency_clone = total_latency.clone();

        let handle = tokio::spawn(async move {
            // 获取信号量许可
            let _permit = semaphore_clone.acquire().await.unwrap();

            // 连接到服务器
            let mut stream = match TcpStream::connect("127.0.0.1:8888").await {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("[Client {}] 连接失败: {}", i, e);
                    return;
                }
            };

            // 等待所有连接就绪
            barrier_clone.wait().await;

            // 发送请求并测量延迟
            for _ in 0..requests_per_conn {
                // 准备请求数据 - 使用随机大小的负载
                let payload = vec![b'A'; 64]; // 固定64字节负载
                let request = DataPack::pack(1, &payload);

                let request_start = Instant::now();

                // 发送请求
                if let Err(e) = stream.write_all(&request).await {
                    eprintln!("[Client {}] 发送请求失败: {}", i, e);
                    break;
                }

                // 读取响应头
                let mut header = [0u8; 8];
                if let Err(e) = stream.read_exact(&mut header).await {
                    eprintln!("[Client {}] 读取响应头失败: {}", i, e);
                    break;
                }

                let (msg_id, data_len) = match DataPack::unpack_header(&header) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("[Client {}] 解析响应头失败: {}", i, e);
                        break;
                    }
                };

                // 读取响应数据
                let mut data = vec![0u8; data_len as usize];
                if let Err(e) = stream.read_exact(&mut data).await {
                    eprintln!("[Client {}] 读取响应数据失败: {}", i, e);
                    break;
                }

                // 计算延迟（微秒）
                let latency = request_start.elapsed().as_micros() as usize;
                latency_clone.fetch_add(latency, Ordering::Relaxed);

                // 增加完成请求计数
                completed_clone.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    // 启动进度报告任务
    let progress_completed = completed_requests.clone();
    let progress_handle = tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            let completed = progress_completed.load(Ordering::Relaxed);
            let progress = (completed as f64 / total_requests as f64) * 100.0;
            println!(
                "[Progress] {:.2}% ({}/{})",
                progress, completed, total_requests
            );

            if completed >= total_requests {
                break;
            }
        }
    });

    // 所有连接已建立，开始测试
    println!("[Client] 所有连接已就绪，开始测试...");
    barrier.wait().await;

    // 等待所有客户端完成
    for handle in handles {
        let _ = handle.await;
    }

    // 停止进度报告
    progress_handle.abort();

    // 计算结果
    let elapsed = start_time.elapsed();
    let completed = completed_requests.load(Ordering::Relaxed);
    let avg_latency = if completed > 0 {
        total_latency.load(Ordering::Relaxed) as f64 / completed as f64
    } else {
        0.0
    };

    // 打印结果
    println!("\n===== 基准测试结果 =====");
    println!("总连接数: {}", connections);
    println!("每连接请求数: {}", requests_per_conn);
    println!("总请求数: {}", total_requests);
    println!("完成请求数: {}", completed);
    println!("总耗时: {:.2} 秒", elapsed.as_secs_f64());
    println!("平均延迟: {:.2} 微秒", avg_latency);
    println!(
        "吞吐量: {:.2} 请求/秒",
        completed as f64 / elapsed.as_secs_f64()
    );

    Ok(())
}
