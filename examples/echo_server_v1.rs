//! # Zerust Echo Server 测试程序
//!
//! 本示例用于验证 Zerust 框架的核心功能：
//! - 异步 TCP 服务器启动与连接处理
//! - 路由分发机制（msg_id -> handler）
//! - 客户端请求发送与响应解析
//! - 服务器的**优雅启动与主动关闭**
//! - 集成测试的完整生命周期控制
//!
//! ✅ 运行方式：
//! ```bash
//! cargo run --example echo_server_v1
//! ```

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use zerust::datapack::DataPack;
use zerust::{DefaultRouter, Response, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ========================================
    // 1. 创建关闭通道：用于外部控制服务器生命周期
    // ========================================
    // 当 shutdown_tx 被 drop 或 send(()) 时，shutdown_rx 将完成
    // server.run() 中通过 tokio::select! 监听该信号，实现优雅退出
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // ========================================
    // 2. 创建并配置路由器
    // ========================================
    let router = Arc::new(DefaultRouter::new());

    // 注册 msg_id = 1 的回显处理函数
    let router_clone = router.clone();
    router_clone.add_route(1, |req| {
        println!("Received echo request: {:?}", req.data());
        Response::new(req.msg_id(), req.data().to_vec()) // 原样返回
    });

    // ========================================
    // 3. 启动服务器（异步任务）
    // ========================================
    let server = Server::new("127.0.0.1:8000", router);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run(shutdown_rx).await {
            eprintln!("[Zerust] Server runtime error: {}", e);
        }
    });

    // ========================================
    // 4. 等待服务器就绪（端口探测）
    // ========================================
    // 替代 sleep()，更可靠：最多等待 5 秒，每 10ms 尝试一次连接
    if let Err(_) = wait_for_server(8000, Duration::from_secs(5)).await {
        eprintln!("[Client] Failed to connect to server within 5 seconds.");
        return Err("Server did not start in time".into());
    }
    println!("[Client] Server is ready. Proceeding with test...");

    // ========================================
    // 5. 运行客户端测试
    // ========================================
    match client().await {
        Ok(()) => println!("✅ Client finished successfully."),
        Err(e) => eprintln!("❌ Client error: {}", e),
    }

    // ========================================
    // 6. 发送关闭信号
    // ========================================
    // 客户端完成，通知服务器关闭
    let _ = shutdown_tx.send(());
    println!("[Main] Shutdown signal sent.");

    // ========================================
    // 7. 等待服务器完全停止
    // ========================================
    // 确保 server.run() 任务完全结束，避免资源泄漏
    let _ = server_handle.await;

    println!("🎉 Program exited gracefully.");
    Ok(())
}

/// 客户端：连接服务器，发送测试请求，接收并验证响应
async fn client() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8000").await?;
    println!("Connected to server");

    // 构造请求：msg_id=1, data="test"
    let bytes = DataPack::pack(1, b"test");
    stream.write_all(&bytes).await?;
    println!("Sent request: msg_id=1, data=test");

    // 读取响应头（8字节）
    let mut header = [0u8; 8];
    stream.read_exact(&mut header).await?;
    let (msg_id, data_len) = DataPack::unpack_header(&header)?;
    println!(
        "Received response header: msg_id={}, data_len={}",
        msg_id, data_len
    );

    // 读取响应数据
    let mut data = vec![0u8; data_len as usize];
    stream.read_exact(&mut data).await?;
    println!(
        "Received response: msg_id={}, data={:?}",
        msg_id,
        String::from_utf8_lossy(&data)
    );

    Ok(())
}

/// 等待服务器在指定端口上启动
///
/// # 参数
/// - `port`: 要探测的端口
/// - `timeout`: 最大等待时间
///
/// # 返回
/// - `Ok(())`: 在超时前成功连接
/// - `Err(_)`: 超时或持续连接失败
async fn wait_for_server(port: u16, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        // 尝试连接，带剩余时间限制
        let connect_fut = TcpStream::connect(("127.0.0.1", port));
        if tokio::time::timeout(deadline - tokio::time::Instant::now(), connect_fut)
            .await
            .is_ok()
        {
            return Ok(()); // 连接成功，退出
        }
        // 短暂休眠后重试
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
