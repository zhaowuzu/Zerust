//! # 服务器核心模块
//!
//! 该模块提供了TCP服务器的核心功能，包括监听连接、处理请求和响应等。
//! 它是框架的主要入口点，负责协调各个组件的工作。
//!
//! ## 主要功能
//!
//! * 绑定并监听TCP端口
//! * 接收客户端连接
//! * 为每个连接创建独立的异步任务
//! * 协调路由器和连接管理器的工作

use std::sync::Arc;
use tokio::net::{TcpStream, TcpListener};
use crate::{error::ZerustError, router::Router, connection::Connection};

/// 表示一个TCP服务器
///
/// `Server` 是框架的主要入口点，负责监听TCP连接并处理客户端请求。
/// 它使用 `Router` 来分发请求，使用 `Connection` 来管理客户端连接。
pub struct Server {
    /// 服务器监听的地址，格式为 "IP:端口"
    addr: String,
    /// 路由器实例，用于分发请求到对应的处理函数
    /// 
    /// 使用 `Arc` 包装，可以在多个线程间安全地共享数据
    router: Arc<dyn Router + Send + Sync>
}

impl Server {
    /// 创建一个新的服务器实例
    ///
    /// # 参数
    /// * `addr` - 服务器监听的地址，格式为 "IP:端口"
    /// * `router` - 路由器实例，用于分发请求到对应的处理函数
    ///
    /// # 返回值
    /// 返回一个新的 `Server` 实例
    pub fn new(addr: &str, router: Arc<dyn Router + Send + Sync>) -> Self {
        Self {
            addr: addr.to_string(),
            router,
        }
    }

    /// 启动服务器并监听指定地址的TCP连接
    ///
    /// 该函数会绑定到配置的地址并开始监听TCP连接，对于每个传入的连接，
    /// 都会创建一个异步任务来处理请求。如果在监听过程中发生IO错误，
    /// 函数会立即返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use zerust::{Server, DefaultRouter, Response, Request};
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 创建路由器
    ///     let router = Arc::new(DefaultRouter::new());
    ///
    ///     // 添加路由处理
    ///     router.add_route(1, |req| {
    ///         println!("Received request: {:?}", req.data());
    ///         Response::new(req.msg_id(), req.data().to_vec())
    ///     });
    ///
    ///     // 启动服务器
    ///     let server = Server::new("127.0.0.1:8080", router);
    ///     server.run().await?
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # 返回值
    ///
    /// * `Ok(())` - 服务器正常启动并运行
    /// * `Err(ZerustError)` - 服务器启动或运行过程中发生错误
    pub async fn run(&self)->Result<(),ZerustError>{
        // 绑定TCP监听器到指定地址
        let listener = TcpListener::bind(&self.addr).await?;
        println!("[Zerust] Server listening on {}", self.addr);

        // 持续接受并处理客户端连接
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // 为每个连接创建独立的异步任务进行处理
                    let router = self.router.clone();
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_connection(stream, router).await {
                            eprintln!("[Zerust] Error handling connection: {}", e);
                        }
                    });
                }
                Err(e)=> return Err(ZerustError::IoError(e))
            }
        }
    }


    /// 处理TCP连接的异步函数
    ///
    /// 该函数负责接收并处理来自客户端的HTTP请求，通过路由器分发请求并返回响应
    ///
    /// # 参数
    /// * `stream` - TCP流连接，用于与客户端进行数据通信
    /// * `router` - 路由器实例，用于处理HTTP请求并生成响应
    ///
    /// # 返回值
    /// * `Result<(), ZerustError>` - 成功时返回空元组，失败时返回Zerust错误
    async fn handle_connection(
        stream: TcpStream,
        router: Arc<dyn Router>,
    )-> Result<(), ZerustError>{
        let mut conn = Connection::new(stream);
        println!("[Zerust] New connection from {:?}", conn.remote_addr());

        // 持续处理来自同一连接的多个请求
        loop {
            // 读取客户端发送的HTTP请求
            let req = match conn.read_request().await{
                Ok(req) => req,
                Err(e) => {
                    println!("[Zerust] Error reading request: {:?}", e);
                    return Err(e);
                }
            };

            // 使用路由器处理请求并生成响应
            let resp = router.handle(&req);

            // 发送HTTP响应给客户端
            conn.send_reponse(resp).await?;
        }
    }

}