//! # 连接管理模块
//!
//! 该模块负责管理TCP连接的生命周期和数据传输，包括读取请求、发送响应等操作。
//! 它是服务器与客户端之间通信的桥梁，处理底层的网络IO操作。

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};
use crate::{error::ZerustError, datapack::DataPack, request::Request, response::Response};
use std::net::SocketAddr;

/// 表示一个TCP连接
///
/// `Connection` 封装了一个TCP流和相关的缓冲区，提供了读取请求和发送响应的方法。
/// 它负责处理底层的网络IO操作，并将原始字节数据转换为应用层的请求和响应对象。
pub struct Connection {
    /// TCP流，用于与客户端进行网络通信
    stream: TcpStream,
    /// 用于存放从流中读取但尚未被应用层处理的数据
    pending_data: Vec<u8>,
}

impl Connection {
    /// 消息头部大小常量，单位为字节
    /// 
    /// 消息头由两部分组成：
    /// * 4字节的消息ID (msg_id)
    /// * 4字节的数据长度 (data_len)
    const HEADER_SIZE: usize = 8; // msg_id(4) + data_len(4)

    /// 创建一个新的连接实例
    ///
    /// # 参数
    /// * `stream` - TCP流，用于与客户端进行网络通信
    ///
    /// # 返回值
    /// 返回一个新的 `Connection` 实例
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            pending_data: Vec::new(),
        }
    }

    /// 获取远程客户端的套接字地址
    ///
    /// 该函数通过底层的流连接获取对端的网络地址信息。
    ///
    /// # 返回值
    ///
    /// * `Ok(SocketAddr)` - 成功获取到的远程套接字地址
    /// * `Err(ZerustError)` - 获取地址失败时返回的错误信息
    ///
    /// # 错误处理
    ///
    /// 当底层IO操作出现错误时，会将IO错误转换为ZerustError::IoError返回
    pub fn remote_addr(&self) ->Result<SocketAddr,ZerustError>{
        // 获取对端地址，如果出现IO错误则转换为ZerustError
        self.stream
            .peer_addr()
            .map_err(ZerustError::IoError)
    }


    /// 从连接中异步读取一个完整的请求消息
    ///
    /// 该函数首先读取固定大小的消息头，解析出消息ID和数据长度，
    /// 然后根据数据长度读取相应的消息体数据，最后构造成Request对象返回。
    ///
    /// # Returns
    ///
    /// * `Result<Request, ZerustError>` - 成功时返回解析出的请求对象，失败时返回错误信息
    ///
    pub async fn read_request(&mut self) -> Result<Request,ZerustError>{
        // 读取消息头
        let header_bytes = self.read_exact(Self::HEADER_SIZE).await?;
        // 解析消息头
        let(msg_id,data_len) = DataPack::unpack_header(&header_bytes)?;
        // 读取消息体
        let data = if data_len > 0 {
            self.read_exact(data_len as usize).await?
        } else {
            Vec::new()
        };
        Ok(Request::new(msg_id,data))
    }


    /// 从流中精确读取指定数量的字节数据
    ///
    /// 该函数会优先从 `pending_data` 中获取数据，如果不够则从流中读取。
    ///
    /// # 参数
    /// * `size` - 需要读取的字节数
    ///
    /// # 返回值
    /// * `Ok(Vec<u8>)` - 成功读取的字节数据
    /// * `Err(ZerustError)` - 读取过程中发生的错误，包括连接关闭等
    async fn read_exact(&mut self, size: usize) -> Result<Vec<u8>, ZerustError> {
        // 首先检查 pending_data 中是否有足够的数据
        while self.pending_data.len() < size {
            // pending_data 中的数据不够，需要从流中读取更多
            let mut buffer = [0u8; 1024]; // 临时缓冲区
            let n = self.stream.read(&mut buffer).await?;
            if n == 0 {
                return Err(ZerustError::ConnectionClosed);
            }
            // 将新读取的数据追加到 pending_data
            self.pending_data.extend_from_slice(&buffer[..n]);
        }

        // 现在 pending_data 中至少有 size 个字节
        let result = self.pending_data.drain(..size).collect(); // 取出前 size 个字节
        Ok(result)
    }


    /// 发送响应消息
    ///
    /// 将响应消息打包并发送到网络流中
    ///
    /// # 参数
    /// * `resp` - 要发送的响应消息
    ///
    /// # 返回值
    /// * `Result<(),ZerustError>` - 发送结果，成功返回Ok(())，失败返回ZerustError错误
    ///
    /// # 异常
    /// * 当网络写入失败时会返回ZerustError错误
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use zerust::{Connection, Response};
    /// use tokio::net::TcpStream;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 连接到服务器
    ///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
    ///     let mut connection = Connection::new(stream);
    ///     
    ///     // 读取请求
    ///     let request = connection.read_request().await?;
    ///     println!("收到请求: 消息ID={}, 数据长度={}", request.msg_id(), request.data().len());
    ///     
    ///     // 创建并发送响应
    ///     let response = Response::new(request.msg_id(), b"Hello, Client!".to_vec());
    ///     connection.send_response(&response).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn send_response(&mut self, resp: &Response) -> Result<(), ZerustError> {
        // 将响应消息打包成字节数据
        let bytes = DataPack::pack(resp.msg_id(), resp.data());
        // 异步写入网络流
        self.stream.write_all(&bytes).await?;
        Ok(())
    }

    /// 从连接中异步读取一个完整的响应消息
    ///
    /// 该函数首先读取固定大小的消息头，解析出消息ID和数据长度，
    /// 然后根据数据长度读取相应的消息体数据，最后构造成Response对象返回。
    ///
    /// # 返回值
    ///
    /// * `Result<Response, ZerustError>` - 成功时返回解析出的响应对象，失败时返回错误信息
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use zerust::{Connection, Request};
    /// use tokio::net::TcpStream;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // 连接到服务器
    ///     let stream = TcpStream::connect("127.0.0.1:8080").await?;
    ///     let mut connection = Connection::new(stream);
    ///     
    ///     // 创建并发送请求
    ///     let request = Request::new(1, b"Hello, Server!".to_vec());
    ///     connection.send_request(&request).await?;
    ///     
    ///     // 读取响应
    ///     let response = connection.read_response().await?;
    ///     println!("收到响应: 消息ID={}, 数据={:?}", response.msg_id(), response.data());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn read_response(&mut self) -> Result<Response, ZerustError> {
        // 读取消息头
        let header_bytes = self.read_exact(Self::HEADER_SIZE).await?;
        // 解析消息头
        let (msg_id, data_len) = DataPack::unpack_header(&header_bytes)?;
        // 读取消息体
        let data = if data_len > 0 {
            self.read_exact(data_len as usize).await?
        } else {
            Vec::new()
        };
        Ok(Response::new(msg_id, data))
    }

    /// 发送请求消息
    ///
    /// 将请求消息打包并发送到网络流中
    ///
    /// # 参数
    /// * `req` - 要发送的请求消息
    ///
    /// # 返回值
    /// * `Result<(), ZerustError>` - 发送结果，成功返回Ok(())，失败返回ZerustError错误
    pub async fn send_request(&mut self, req: &Request) -> Result<(), ZerustError> {
        // 将请求消息打包成字节数据
        let bytes = DataPack::pack(req.msg_id(), req.data());
        // 异步写入网络流
        self.stream.write_all(&bytes).await?;
        Ok(())
    }
}