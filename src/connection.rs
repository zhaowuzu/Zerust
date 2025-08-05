/*连接管理*/

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use crate::{error::ZerustError, datapack::DataPack, request::Request, response::Response};
use std::net::SocketAddr;

pub struct Connection {
    stream : TcpStream,
    pending_data:Vec<u8>, // 用于存放从流中读取但尚未被应用层处理的数据
}

impl Connection {
    const HEADER_SIZE : usize = 8; // msg_id(4) + data_len(4)

    pub fn new(stream: TcpStream)-> Self {
        Self {
            stream,
            pending_data:Vec::new(),
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
    pub async fn send_reponse(&mut self,resp:Response) -> Result<(),ZerustError> {
        // 将响应消息打包成字节数据
        let bytes = DataPack::pack(resp.msg_id(),resp.data());
        // 异步写入网络流
        self.stream.write_all(&bytes).await?;
        Ok(())
    }

}