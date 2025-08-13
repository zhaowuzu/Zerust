// 简单的Zinx客户端示例
package main

import (
	"fmt"
	"github.com/aceld/zinx/ziface"
	"github.com/aceld/zinx/znet"
	"time"
)

// 客户端消息处理函数
func pingLoop(conn ziface.IConnection) {
	for i := 0; i < 5; i++ {
		// 发送消息
		err := conn.SendMsg(1, []byte("Hello Zinx Server"))
		if err != nil {
			fmt.Println("发送消息失败:", err)
			break
		}
		fmt.Println("发送消息成功")
		time.Sleep(1 * time.Second)
	}
}

// 连接建立时的回调
func onClientStart(conn ziface.IConnection) {
	fmt.Println("连接建立成功")
	go pingLoop(conn)
}

func main() {
	// 创建客户端
	client := znet.NewClient("127.0.0.1", 8999)
	
	// 设置连接建立的回调
	client.SetOnConnStart(onClientStart)
	
	// 启动客户端
	client.Start()
	
	// 防止程序退出
	select {}
}