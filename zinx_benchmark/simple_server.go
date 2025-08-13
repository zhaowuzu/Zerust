// 简单的Zinx服务器示例
package main

import (
	"fmt"
	"github.com/aceld/zinx/ziface"
	"github.com/aceld/zinx/znet"
)

// 回显路由
type EchoRouter struct {
	znet.BaseRouter
}

// 处理消息
func (r *EchoRouter) Handle(request ziface.IRequest) {
	// 打印收到的消息
	fmt.Println("收到消息:", string(request.GetData()))
	
	// 回复消息
	request.GetConnection().SendMsg(request.GetMsgID(), request.GetData())
}

func main() {
	// 创建服务器
	s := znet.NewServer()

	// 注册路由
	s.AddRouter(1, &EchoRouter{})

	// 启动服务器
	fmt.Println("服务器启动在 127.0.0.1:8999")
	s.Serve()
}