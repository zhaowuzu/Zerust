/*错误处理*/

use std::io;
use thiserror::Error;

#[derive(Error, Debug)] // 为 ZerustError 自动完成Error, Display, Debug 等 trait 的实现
pub enum ZerustError{

    // #[from] 实现了从 io::Error 到 ZerustError 的自动转换
    #[error("I/O error:{0}")]
    IoError(#[from] io::Error),

    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    #[error("Invalid header format")]
    InvalidHeader,

    #[error("Protocol error:{0}")]
    ProtocolError(String),
}