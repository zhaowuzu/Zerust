//! # Zerust
//! 
//! Zerust 是一个高性能、零成本的 Rust 网络框架，灵感来源于 Go 语言的 Zinx 框架。
//! 它提供了简单易用的 API，用于构建高效的网络应用程序。
//! 
//! ## 框架特性
//! 
//! * **高性能**：基于 Tokio 异步运行时，提供卓越的并发性能
//! * **简单易用**：提供直观的 API，易于上手和使用
//! * **可扩展**：模块化设计，便于扩展和定制
//! * **零成本抽象**：遵循 Rust 的零成本抽象原则，无额外运行时开销
//! 
//! ## 模块结构
//! 
//! * `error` - 错误处理模块，定义框架中可能出现的各种错误类型
//! * `request` - 请求封装模块，处理客户端发送的请求数据
//! * `response` - 响应封装模块，处理服务器返回的响应数据
//! * `router` - 路由系统模块，负责根据消息ID分发请求到对应的处理函数
//! * `datapack` - 协议编解码模块，处理数据的打包和解包
//! * `connection` - 连接管理模块，处理TCP连接的生命周期和数据传输
//! * `server` - 服务器核心模块，提供TCP服务器的基本功能
//! 
//! ## 示例
//! 
//! ### 服务器示例
//! 
//! 以下是一个简单的回显服务器示例，它接收客户端发送的消息并原样返回：
//! 
//! ```rust
//! use zerust::{Server, DefaultRouter, Response, Request};
//! use std::sync::Arc;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建路由器
//!     let router = Arc::new(DefaultRouter::new());
//! 
//!     // 添加路由处理
//!     router.add_route(1, |req| {
//!         println!("Received echo request: {:?}", req.data());
//!         Response::new(req.msg_id(), req.data().to_vec())
//!     });
//! 
//!     // 启动服务器
//!     let server = Server::new("127.0.0.1:8080", router);
//!     server.run().await?;
//! 
//!     Ok(())
//! }
//! ```
//! 
//! ### 客户端示例
//! 
//! 以下是一个简单的客户端示例，它连接到服务器并发送消息：
//! 
//! ```rust
//! use tokio::io::{AsyncReadExt, AsyncWriteExt};
//! use tokio::net::TcpStream;
//! use zerust::datapack::DataPack;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
//! 
//!     // 发送消息（msg_id = 1, data="test"）
//!     let bytes = DataPack::pack(1, b"test");
//!     stream.write_all(&bytes).await?;
//!     println!("Sent request: msg_id=1, data=test");
//!     
//!     // 读取响应
//!     let mut header = [0u8; 8];
//!     stream.read_exact(&mut header).await?;
//!     let (msg_id, data_len) = DataPack::unpack_header(&header)?;
//!     println!("Received response header: msg_id={}, data_len={}", msg_id, data_len);
//!     
//!     let mut data = vec![0u8; data_len as usize];
//!     stream.read_exact(&mut data).await?;
//!     println!("Received response: msg_id={}, data={:?}", msg_id, data);
//! 
//!     Ok(())
//! }
//! ```
//! 
//! 更多示例请参考 `examples` 目录中的代码。

// 导出各个模块
pub mod error;
pub mod request;
pub mod response;
pub mod router;
pub mod datapack;
pub mod connection;
pub mod server;

// 重新导出常用的类型，方便用户直接使用
pub use error::ZerustError;
pub use request::Request;
pub use response::Response;
pub use router::{Router,DefaultRouter};
pub use server::Server;