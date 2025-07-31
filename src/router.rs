/*路由系统*/

use crate::request::Request;
use crate::response::Response;
use dashmap::DashMap;
use std::sync::Arc;

// Send 超trait约束：表示该类型的所有权可以在不同的线程间安全转移
// Sync 超trait约束：表示该类型的引用（&T）可以在多个线程间安全共享
pub trait Router: Send + Sync {
    fn handle(&self,req:&Request) -> Response;
}

// Handler 是一个指向实现了 Fn(&Request) -> Response 且满足 Send + Sync 约束的闭包或函数的堆分配指针。
// 它代表了处理特定请求的逻辑。
pub type Handler = Box<dyn Fn(&Request) -> Response + Send + Sync>;

pub struct DefaultRouter {
    routes:DashMap<u32,Handler>
}
impl DefaultRouter {
    pub fn new() ->Self{
        Self{
            routes:DashMap::new(),
        }
    }

    pub fn add_route<F>(&self,msg_id:u32,handler:F)
    where
        // 'static: F 类型不能包含任何非 'static 的引用。这确保了闭包捕获的任何数据都拥有所有权或具有 'static 生命周期，使得 Handler 可以安全地在程序的整个生命周期内存在
        F:Fn(&Request) -> Response + Send + Sync + 'static,
    {
        self.routes.insert(msg_id,Box::new(handler));
    }
}

impl Router for DefaultRouter {
    fn handle(&self, req: &Request) -> Response {
        match self.routes.get(&req.msg_id()){
            Some(handler) => handler(req),
            None => Response::not_found(),
        }
    }
}