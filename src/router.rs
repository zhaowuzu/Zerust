//! # 路由系统模块
//!
//! 该模块定义了请求路由的接口和默认实现，负责将请求根据消息ID分发到对应的处理函数。
//! 路由系统是框架的核心组件之一，它允许用户注册自定义的请求处理逻辑。

use crate::request::Request;
use crate::response::Response;
use dashmap::DashMap;

/// 路由器接口
///
/// 定义了路由器的基本行为，即根据请求生成响应。
/// 实现了 `Send` 和 `Sync` trait，使其可以在多线程环境中安全使用。
///
/// * `Send` 超trait约束：表示该类型的所有权可以在不同的线程间安全转移
/// * `Sync` 超trait约束：表示该类型的引用（&T）可以在多个线程间安全共享
pub trait Router: Send + Sync {
    /// 处理请求并生成响应
    ///
    /// # 参数
    /// * `req` - 请求对象的引用
    ///
    /// # 返回值
    /// 返回对应的响应对象
    fn handle(&self, req: &Request) -> Response;
}

/// 请求处理函数类型
///
/// `Handler` 是一个指向实现了 `Fn(&Request) -> Response` 且满足 `Send + Sync` 约束的闭包或函数的堆分配指针。
/// 它代表了处理特定请求的逻辑。
pub type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

/// 默认路由器实现
///
/// 使用 `DashMap` 存储消息ID到处理函数的映射，支持并发访问。
/// `DashMap` 是一个线程安全的哈希表，适合在多线程环境中使用。
pub struct DefaultRouter {
    /// 存储消息ID到处理函数的映射
    routes: DashMap<u32, Handler>,
}

impl DefaultRouter {
    /// 创建一个新的默认路由器实例
    ///
    /// # 返回值
    /// 返回一个新的 `DefaultRouter` 实例，其中包含一个空的路由表
    ///
    /// # 示例
    ///
    /// ```rust
    /// use zerust::{DefaultRouter, Response, Request};
    /// use std::sync::Arc;
    ///
    /// // 创建路由器
    /// let router = Arc::new(DefaultRouter::new());
    ///
    /// // 添加路由处理
    /// router.add_route(1, |req| {
    ///     println!("处理消息ID为1的请求");
    ///     Response::new(req.msg_id(), b"Hello, World!".to_vec())
    /// });
    ///
    /// // 添加另一个路由处理
    /// router.add_route(2, |req| {
    ///     println!("处理消息ID为2的请求");
    ///     Response::new(req.msg_id(), b"Echo: ".iter().chain(req.data().iter()).cloned().collect())
    /// });
    /// ```
    pub fn new() -> Self {
        Self {
            routes: DashMap::new(),
        }
    }

    /// 添加路由规则
    ///
    /// 将指定的消息ID与处理函数关联起来，当收到对应消息ID的请求时，
    /// 会调用该处理函数生成响应。
    ///
    /// # 参数
    /// * `msg_id` - 消息ID
    /// * `handler` - 处理函数，接收请求对象的引用，返回响应对象
    ///
    /// # 类型参数
    /// * `F` - 处理函数的类型，必须实现 `Fn(&Request) -> Response + Send + Sync + 'static`
    ///   * `'static` 约束确保了闭包捕获的任何数据都拥有所有权或具有 'static 生命周期，
    ///     使得 Handler 可以安全地在程序的整个生命周期内存在
    pub fn add_route<F>(&self, msg_id: u32, handler: F)
    where
        F: Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.routes.insert(msg_id, Box::new(handler));
    }
}

/// 为 `DefaultRouter` 实现 `Default` trait
impl Default for DefaultRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// 为 `DefaultRouter` 实现 `Router` trait
impl Router for DefaultRouter {
    /// 处理请求并生成响应
    ///
    /// 根据请求的消息ID查找对应的处理函数，如果找到则调用该函数处理请求，
    /// 否则返回一个表示路由未找到的响应。
    ///
    /// # 参数
    /// * `req` - 请求对象的引用
    ///
    /// # 返回值
    /// 返回对应的响应对象
    fn handle(&self, req: &Request) -> Response {
        match self.routes.get(&req.msg_id()) {
            Some(handler) => handler(req),
            None => Response::not_found(),
        }
    }
}
