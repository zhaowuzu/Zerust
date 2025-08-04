/*服务器核心*/
/*
TCP 服务器基础
主要开启TCP监听端口，接收客户端连接
*/
use std::sync::Arc;
use tokio::net::{TcpStream, TcpListener};
use crate::{error::ZerustError, router::Router, connection::Connection};

pub struct Server{
    addr : String,
    router :Arc<dyn Router+Send+Sync> // Arc 可以在多个线程建安全的共享数据，在这里主要是共享 Router 实例
}

impl Server{
    pub fn new(addr:&str,router:Arc<dyn Router+Send+Sync>) -> Self{
        Self{
            addr:addr.to_string(),
            router,
        }
    }

    /// 启动服务器并监听指定地址的TCP连接
    ///
    /// 该函数会绑定到配置的地址并开始监听TCP连接，对于每个传入的连接，
    /// 都会创建一个异步任务来处理请求。如果在监听过程中发生IO错误，
    /// 函数会立即返回错误。
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