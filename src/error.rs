//! # 错误处理模块
//!
//! 该模块定义了框架中可能出现的各种错误类型，并提供了错误处理的功能。
//! 使用 `thiserror` 库简化了错误类型的定义和处理。

use std::io;
use thiserror::Error;

/// Zerust 框架的错误类型
///
/// 该枚举包含了框架中可能出现的所有错误类型，包括IO错误、连接关闭、
/// 无效的消息头格式以及协议错误等。通过实现 `Error` 和 `Debug` trait，
/// 使得错误可以方便地打印和处理。
#[derive(Error, Debug)] // 为 ZerustError 自动完成Error, Display, Debug 等 trait 的实现
pub enum ZerustError{

    /// IO错误，包装了标准库中的 `io::Error`
    ///
    /// 当底层IO操作（如网络读写）失败时会返回此错误。
    /// \#\[from\] 属性实现了从 io::Error 到 ZerustError 的自动转换
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// 连接意外关闭错误
    ///
    /// 当客户端连接在预期之外关闭时会返回此错误。
    #[error("Connection closed unexpectedly")]
    ConnectionClosed,

    /// 无效的消息头格式错误
    ///
    /// 当解析消息头时发现格式不符合预期时会返回此错误。
    #[error("Invalid header format")]
    InvalidHeader,

    /// 协议错误，包含错误描述信息
    ///
    /// 当消息不符合协议规范时会返回此错误，附带具体的错误描述。
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}