//! # 响应封装模块
//!
//! 该模块定义了服务器响应的数据结构和相关方法，用于表示服务器对客户端请求的响应。
//! 响应包含消息ID和响应数据两部分，消息ID通常与请求的消息ID对应。

/// 表示服务器返回的响应
///
/// 响应包含两个主要部分：
/// * `msg_id` - 消息ID，通常与请求的消息ID对应
/// * `data` - 响应携带的数据，以字节数组形式存储
///
/// 实现了 `Debug` trait，方便调试和日志记录。
#[derive(Debug)]
pub struct Response {
    /// 消息ID，通常与请求的消息ID对应
    msg_id: u32,
    /// 响应携带的数据
    data: Vec<u8>,
}

impl Response {
    /// 创建一个新的响应实例
    ///
    /// # 参数
    /// * `msg_id` - 消息ID，通常与请求的消息ID对应
    /// * `data` - 响应携带的数据
    ///
    /// # 返回值
    /// 返回一个新的 `Response` 实例
    pub fn new(msg_id: u32, data: Vec<u8>) -> Self {
        Self { msg_id, data }
    }

    /// 创建一个表示路由未找到的响应
    ///
    /// 当请求的消息ID没有对应的处理函数时，返回此响应。
    /// 使用404作为消息ID，响应数据为"Route not found"。
    ///
    /// # 返回值
    /// 返回一个表示路由未找到的 `Response` 实例
    pub fn not_found() -> Self {
        Self::new(404, b"Route not found".to_vec())
    }

    /// 获取响应的消息ID
    ///
    /// # 返回值
    /// 返回响应的消息ID
    pub fn msg_id(&self) -> u32 {
        self.msg_id
    }

    /// 获取响应携带的数据
    ///
    /// # 返回值
    /// 返回响应携带的数据的引用
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
