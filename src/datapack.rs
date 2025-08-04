/*协议编解码
协议格式：假设消息由一个 8 字节的头部和一个可变长度的数据部分组成。
头部 (8 bytes)：
前 4 字节：msg_id (u32, Little-Endian)
后 4 字节：data_len (u32, Little-Endian)，表示后续数据的字节长度。
数据部分：紧接着头部，长度为 data_len 字节的原始数据。
*/

use crate::error::ZerustError;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Cursor};

pub struct DataPack;

impl DataPack {
    /// 解包消息头信息
    ///
    /// 从给定的字节切片中读取消息ID和数据长度信息
    ///
    /// # 参数
    /// * `header` - 包含消息头信息的字节切片
    ///
    /// # 返回值
    /// 返回Result类型，成功时包含(msg_id, data_len)元组，失败时返回ZerustError错误
    /// * `msg_id` - 消息ID
    /// * `data_len` - 数据长度
    ///
    /// # 错误处理
    /// 当字节切片长度不足或格式不正确时，会返回相应的ZerustError错误
    pub fn unpack_header(header:&[u8])->Result<(u32,u32),ZerustError>{
        // 创建游标用于读取字节数据
        let mut cursor = Cursor::new(header);
        // 以小端序读取消息ID和数据长度
        let msg_id = cursor.read_u32::<LittleEndian>()?;
        let data_len = cursor.read_u32::<LittleEndian>()?;
        Ok((msg_id,data_len))
    }


    /// 将消息ID和数据打包成字节向量
    ///
    /// 该函数按照特定协议格式将消息ID和数据封装成一个字节向量，
    /// 格式为：消息ID(4字节)+数据长度(4字节)+数据内容
    ///
    /// # 参数
    /// * `msg_id` - 消息ID，32位无符号整数
    /// * `data` - 要打包的数据切片
    ///
    /// # 返回值
    /// 返回包含打包后数据的字节向量
    pub fn pack(msg_id:u32,data:&[u8])-> Vec<u8>{
        // 创建缓冲区，容量为头部8字节加上数据长度
        let mut buf = Vec::with_capacity(8+data.len());
        // 写入消息ID，使用小端序
        buf.write_u32::<LittleEndian>(msg_id).unwrap();
        // 写入数据长度，使用小端序
        buf.write_u32::<LittleEndian>(data.len() as u32).unwrap();
        // 追加数据内容
        buf.extend_from_slice( data);
        buf
    }

}
