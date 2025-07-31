/*响应封装*/

#[derive(Debug)]
pub struct Response {
    msg_id:u32, // 响应消息id
    data:Vec<u8>, // 响应数据
}
impl Response {
    pub fn new(msg_id:u32,data:Vec<u8>) ->Self{
        Self{msg_id,data}
    }
    pub fn not_found() ->Self{
        Self::new(404,b"Route not found".to_vec())
    }
    pub fn msg_id(&self) ->u32{
        self.msg_id
    }
    pub fn data(&self) ->&[u8]{
        &self.data
    }
}
