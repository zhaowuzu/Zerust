use zerust::{Server,DefaultRouter,Response};
use std::sync::Arc;

/*
测试方法：
启动服务器：cargo run --example echo_server
*/

#[tokio::main]
async fn main() ->Result<(),Box<dyn std::error::Error>>{
    // 创建路由器
    let router = Arc::new(DefaultRouter::new());

    // 注册路由处理程序
    router.add_route(1,|req|{
       println!("Received echo request: {:?}", req.data());
        Response::new(req.msg_id(),req.data().to_vec())
    });

    // 启动服务器
    let server = Server::new("127.0.0.1:8080",router);
    server.run().await?;

    Ok(())
}