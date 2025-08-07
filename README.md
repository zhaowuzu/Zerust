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
zerust = "1.0.0"
```

## 使用示例

### 简单服务器

```rust
use zerust::{Server, DefaultRouter, Response, Request};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建路由器
    let router = Arc::new(DefaultRouter::new());

    // 添加路由处理
    router.add_route(1, |req| {
        println!("Received echo request: {:?}", req.data());
        Response::new(req.msg_id(), req.data().to_vec())
    });

    // 启动服务器
    let server = Server::new("127.0.0.1:8080", router);
    server.run().await?

    Ok(())
}
```

### 客户端示例

```rust
use zerust::{Request, Connection};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 连接服务器
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut connection = Connection::new(stream);
    
    // 创建并发送请求
    let request = Request::new(1, b"Hello, Zerust!".to_vec());
    connection.send_request(&request).await?;
    
    // 接收响应
    let response = connection.read_response().await?;
    println!("Received response: {:?}", response.data());
    
    Ok(())
}
```

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

