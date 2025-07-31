/*请求封装*/

#[derive(Debug)]
pub struct Request {
    msg_id:u32, // 消息id
    data:Vec<u8>, // 消息数据
}

impl Request {
    pub fn new(msg_id:u32,data:Vec<u8>) ->Self{
        Self{msg_id,data}
    }
    pub fn msg_id(&self) ->u32{
        self.msg_id
    }
    pub fn data(&self) ->&[u8]{
        &self.data
    }
}
