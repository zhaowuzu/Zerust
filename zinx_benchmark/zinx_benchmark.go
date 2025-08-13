// Zinx 性能测试程序
package main

import (
	"fmt"
	"sync"
	"sync/atomic"
	"time"

	"github.com/aceld/zinx/ziface"
	"github.com/aceld/zinx/znet"
)

// 回显路由
type EchoRouter struct {
	znet.BaseRouter
}

// 处理消息
func (r *EchoRouter) Handle(request ziface.IRequest) {
	// 直接返回收到的数据
	request.GetConnection().SendMsg(request.GetMsgID(), request.GetData())
}

// 运行客户端测试
func runBenchmark(connections, requestsPerConn int) {
	// 启动服务器
	go func() {
		// 创建服务器，使用自定义配置
		// 创建服务器
		s := znet.NewServer()

		// 注册路由
		s.AddRouter(1, &EchoRouter{})

		// 启动服务器
		fmt.Println("[Server] 基准测试服务器启动在 127.0.0.1:9999")
		s.Serve()
	}()

	// 等待服务器启动
	time.Sleep(1 * time.Second)

	fmt.Printf("[Client] 开始基准测试: %d 并发连接, 每连接 %d 请求\n", connections, requestsPerConn)

	// 统计数据
	totalRequests := connections * requestsPerConn
	completedRequests := atomic.Int64{}
	totalLatency := atomic.Int64{}

	// 同步屏障，确保所有连接同时开始
	var wg sync.WaitGroup
	var barrier sync.WaitGroup
	barrier.Add(1)

	// 启动客户端连接
	wg.Add(connections)
	startTime := time.Now()

	for i := 0; i < connections; i++ {
		go func(id int) {
			defer wg.Done()

			// 连接到服务器
			client := znet.NewClient("127.0.0.1", 9999)
			if client == nil {
				fmt.Printf("[Client %d] 连接失败\n", id)
				return
			}
			
			// 启动客户端
			client.Start()
			defer client.Stop()
			
			// 等待连接建立
			time.Sleep(100 * time.Millisecond)

			// 等待所有连接就绪
			barrier.Wait()

			// 发送请求
			for j := 0; j < requestsPerConn; j++ {
				// 准备64字节负载
				payload := make([]byte, 64)
				for k := range payload {
					payload[k] = 'A'
				}

				// 测量延迟
				requestStart := time.Now()

				// 获取连接对象
				conn := client.Conn()
				if conn == nil {
					fmt.Printf("[Client %d] 获取连接失败\n", id)
					continue
				}
				
				// 发送消息
				err := conn.SendMsg(1, payload)
				if err != nil {
					fmt.Printf("[Client %d] 请求失败: %s\n", id, err)
					continue
				}

				// 计算延迟（微秒）
				latency := time.Since(requestStart).Microseconds()
				totalLatency.Add(latency)

				// 增加完成请求计数
				completedRequests.Add(1)
			}
		}(i)
	}

	// 启动进度报告
	go func() {
		for {
			time.Sleep(1 * time.Second)
			completed := completedRequests.Load()
			progress := float64(completed) / float64(totalRequests) * 100.0
			fmt.Printf("[Progress] %.2f%% (%d/%d)\n", progress, completed, totalRequests)

			if completed >= int64(totalRequests) {
				break
			}
		}
	}()

	// 所有连接已建立，开始测试
	fmt.Println("[Client] 所有连接已就绪，开始测试...")
	barrier.Done()

	// 等待所有客户端完成
	wg.Wait()

	// 计算结果
	elapsed := time.Since(startTime)
	completed := completedRequests.Load()
	avgLatency := float64(0)
	if completed > 0 {
		avgLatency = float64(totalLatency.Load()) / float64(completed)
	}

	// 打印结果
	fmt.Println("\n===== 基准测试结果 =====")
	fmt.Printf("总连接数: %d\n", connections)
	fmt.Printf("每连接请求数: %d\n", requestsPerConn)
	fmt.Printf("总请求数: %d\n", totalRequests)
	fmt.Printf("完成请求数: %d\n", completed)
	fmt.Printf("总耗时: %.2f 秒\n", elapsed.Seconds())
	fmt.Printf("平均延迟: %.2f 微秒\n", avgLatency)
	fmt.Printf("吞吐量: %.2f 请求/秒\n", float64(completed)/elapsed.Seconds())
}

func main() {
	// 使用固定参数
	connections := 100
	requestsPerConn := 1000

	// 运行基准测试
	runBenchmark(connections, requestsPerConn)
}