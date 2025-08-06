//! # 请求封装模块
//!
//! 该模块定义了客户端请求的数据结构和相关方法，用于在服务器端表示和处理客户端发送的请求。
//! 请求包含消息ID和消息数据两部分，消息ID用于路由到对应的处理函数。

/// 表示客户端发送的请求
///
/// 请求包含两个主要部分：
/// * `msg_id` - 消息ID，用于标识请求类型并路由到对应的处理函数
/// * `data` - 请求携带的数据，以字节数组形式存储
///
/// 实现了 `Debug` trait，方便调试和日志记录。
#[derive(Debug)]
pub struct Request {
    /// 消息ID，用于标识请求类型
    msg_id: u32,
    /// 请求携带的数据
    data: Vec<u8>,
}

impl Request {
    /// 创建一个新的请求实例
    ///
    /// # 参数
    /// * `msg_id` - 消息ID，用于标识请求类型
    /// * `data` - 请求携带的数据
    ///
    /// # 返回值
    /// 返回一个新的 `Request` 实例
    pub fn new(msg_id: u32, data: Vec<u8>) -> Self {
        Self { msg_id, data }
    }

    /// 获取请求的消息ID
    ///
    /// # 返回值
    /// 返回请求的消息ID
    pub fn msg_id(&self) -> u32 {
        self.msg_id
    }

    /// 获取请求携带的数据
    ///
    /// # 返回值
    /// 返回请求携带的数据的引用
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
