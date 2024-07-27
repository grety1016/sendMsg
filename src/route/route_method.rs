//引入JWT模块
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use rocket::{
    fairing::{self, Fairing},
    http::uri::Absolute,
    request::Outcome,
    Data, Request, Response,
};

//Hash加密库:
pub use crypto::{digest::Digest, sha2::Sha256};

//引入系统时间
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize}; //用于结构体上方的系列化宏

use std::env;

/// Token验证Fairing
pub struct TokenFairing;
//Token Fairing实现
#[rocket::async_trait]
impl Fairing for TokenFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Token validation",
            kind: fairing::Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>){ 
        let token = req.headers().get_one("Authorization");
        println!("req1:{:#?}", token);
        
    }
}

//创建JWT结构体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LoginUser {
    pub userName: String,
    pub userPwd: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    userName: String,
    userPwd: String,
    token: String,
    code: i32, // 0：成功，非0：失败
    errmsg: String,
}
impl LoginResponse {
    pub fn new(data: LoginUser, code: i32, token: String, errmsg: String) -> Self {
        LoginResponse {
            code,
            token,
            userName: data.userName,
            userPwd: "".to_owned(),
            errmsg,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

impl Claims {
    pub fn new(sub: String) -> Self {
        let nowTimeStamp = chrono::Local::now().timestamp() as usize;
        let exp = nowTimeStamp + 30; //7 * 24 * 60 * 60; //设置token过期时间为一周
        Claims { sub, exp }
    }

    pub fn get_token(usrPhone: String) -> String {
        let secretKey = env::var("TokenSecretKey").unwrap_or_else(|_| String::from("kephi520."));

        let mut hasherSecretKey = Sha256::new();
        hasherSecretKey.input_str(secretKey.as_ref());
        let hex = hasherSecretKey.result_str();

        let claims = Claims::new(usrPhone.to_owned());
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secretKey.as_ref()),
        )
        .unwrap();
        token
    }
    pub fn verify_token(token: String) {
        let secretKey = env::var("TokenSecretKey").unwrap_or_else(|_| String::from("kephi520."));

        let detoken = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secretKey.as_ref()),
            &Validation::default(),
        );
    }
}


