# Zerust

[![Crates.io](https://img.shields.io/crates/v/zerust.svg)](https://crates.io/crates/zerust)
[![Documentation](https://docs.rs/zerust/badge.svg)](https://docs.rs/zerust)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Zerust 是一个轻量级的 Rust 网络框架，灵感来源于 Go 语言的 Zinx 框架。它提供了简单易用的 API 和高效的网络通信功能。

## 特性

- **异步处理**：基于 Tokio 异步运行时，提供高性能的并发处理
- **简单接口**：简洁明了的 API 设计，易于使用
- **可扩展**：灵活的路由系统，支持自定义处理器
- **类型安全**：利用 Rust 的类型系统提供编译时安全检查
- **错误处理**：完善的错误处理机制和详细的错误信息
- **文档完善**：详尽的文档注释，支持 `cargo doc` 生成完整的 API 文档

## 安装

在你的项目的 `Cargo.toml` 文件中添加：

```toml
[dependencies]
zerust = "1.0.3"
```

## 使用示例

### Echo 服务器示例

以下是一个完整的回显服务器示例，包含服务器启动、路由配置、客户端测试和优雅关闭的完整生命周期：

```rust
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use zerust::datapack::DataPack;
use zerust::{DefaultRouter, Response, Server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建关闭通道：用于外部控制服务器生命周期
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // 创建并配置路由器
    let router = Arc::new(DefaultRouter::new());

    // 注册 msg_id = 1 的回显处理函数
    let router_clone = router.clone();
    router_clone.add_route(1, |req| {
        println!("Received echo request: {:?}", req.data());
        Response::new(req.msg_id(), req.data().to_vec()) // 原样返回
    });

    // 启动服务器（异步任务）
    let server = Server::new("127.0.0.1:8000", router);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.run(shutdown_rx).await {
            eprintln!("[Zerust] Server runtime error: {}", e);
        }
    });

    // 等待服务器就绪
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("[Client] Server is ready. Proceeding with test...");

    // 客户端测试
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
    
    // 读取响应数据
    let mut data = vec![0u8; data_len as usize];
    stream.read_exact(&mut data).await?;
    println!(
        "Received response: msg_id={}, data={:?}",
        msg_id,
        String::from_utf8_lossy(&data)
    );

    // 发送关闭信号
    let _ = shutdown_tx.send(());
    println!("[Main] Shutdown signal sent.");

    // 等待服务器完全停止
    let _ = server_handle.await;

    println!("🎉 Program exited gracefully.");
    Ok(())
}
```

完整示例代码可在 `examples/echo_server_v1.rs` 中找到，运行方式：

```bash
cargo run --example echo_server_v1
```

## 性能测试结果

### 测试环境
- 操作系统：Windows 10 Pro
- CPU：Intel Core i7-10700 @ 2.90GHz
- 内存：16GB
- 测试工具：自定义基准测试客户端和服务器

### Zerust 性能
- 吞吐量：217,391.30 请求/秒
- 平均延迟：92.00 微秒
- 总耗时：0.46 秒
- 完成率：100%

### Zinx 性能
- 吞吐量：294,697.33 请求/秒
- 平均延迟：72.89 微秒
- 总耗时：0.34 秒
- 完成率：100%

### 对比分析
- Zinx 在原始性能上略优于 Zerust
- Zerust 在开发体验和易用性方面更佳
- Zerust 提供了更符合 Rust 生态系统的 API 设计

## 文档

详细的 API 文档可以通过以下命令生成：

```bash
cargo doc --open
```
也可以参考已发布的文档：[https://crates.io/crates/zerust](https://crates.io/crates/zerust)

所有的源代码文件都包含详细的文档注释，包括模块、结构体、方法和函数的说明，以及使用示例。

## 许可证

本项目采用 MIT 许可证 - 详情请参阅 [LICENSE](LICENSE) 文件。


🤝 我们遵循 Contributor Covenant 行为准则，欢迎所有贡献者。

