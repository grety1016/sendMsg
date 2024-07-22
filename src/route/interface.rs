use serde::{Deserialize, Serialize}; //用于结构体上方的系列化宏

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginUser {
    pub userName: String,
    pub userPwd: String,
    pub code: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    userName: String,
    userPwd: String,
    pub code: i32, // 0：成功，非0：失败
}
impl LoginResponse {
    pub fn new(data: LoginUser, code: i32) -> Self {
        LoginResponse {
            code,
            userName: data.userName,
            userPwd: data.userPwd,
        }
    }
}
