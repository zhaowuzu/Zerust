use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use zerust::datapack::DataPack;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    // 发送消息（msg_id = 1,data="test"）
    let bytes = DataPack::pack(1, b"test");
    stream.write_all(&bytes).await?;
    println!("Sent request: msg_id=1, data=test");
    // 读取响应
    let mut header = [0u8; 8];
    stream.read_exact(&mut header).await?;
    let (msg_id, data_len) = DataPack::unpack_header(&header)?;
    println!(
        "Received response header: msg_id={}, data_len={}",
        msg_id, data_len
    );
    let mut data = vec![0u8; data_len as usize];
    stream.read_exact(&mut data).await?;

    println!("Received response: msg_id={}, data={:?}", msg_id, data);

    Ok(())
}
