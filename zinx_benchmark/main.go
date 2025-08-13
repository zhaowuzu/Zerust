// Zinx 基准测试主程序
package main

import (
	"flag"
)

func main() {
	// 解析命令行参数
	serverMode := flag.Bool("server", false, "运行服务器模式")
	connections := flag.Int("connections", 100, "并发连接数")
	requestsPerConn := flag.Int("requests", 1000, "每连接请求数")
	flag.Parse()

	if *serverMode {
		runServer()
	} else {
		runClient(*connections, *requestsPerConn)
	}
}