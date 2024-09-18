//引入JWT模块
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use rocket::{
    data::{Data, ToByteUnit},
    fairing::{self, Fairing, Kind},
    http::{uri::Origin, Method},
    request::Outcome,
    FromForm, Request, Response,
};

//Hash加密库:
pub use crypto::{digest::Digest, sha2::Sha256};

use serde::{Deserialize, Serialize};
use tracing::field; //用于结构体上方的系列化宏

use std::{borrow::Cow, env};

/// Token验证Fairinge
pub struct TokenFairing;
//Token Fairing实现
#[rocket::async_trait]
impl Fairing for TokenFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Token validation",
            kind: Kind::Request | Kind::Response,
        }
    }
    //更新测试
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // println!("{}", req.uri().path());
        // println!("{}", req.uri());

        let uri = req.uri().to_string().to_lowercase();
        let url = Origin::try_from(uri).unwrap();
        req.set_uri(url);
        // println!("{}", url);
        // println!("{:#?}", req);
        // println!("{:#?}\n{:#?}\n{:#?}\n{:#?}", req.uri(), req.method(), req.headers().to_owned(),req.to_string());
        /*************************************************************************************
        如下代码用于验证token，并且是POST方法才生效*/
        if req.uri().to_string() == "/user/login" && req.method() == Method::Post {
            let token = req.headers().get_one("Authorization");
            let mut verifyResult: bool = false;
            if let Some(value) = token {
                verifyResult = Claims::verify_token(value.to_string()).await;
            }

            if verifyResult == true {
                // println!("验证成功");
                return;
            } else {
                if req.uri().to_string() == "/user/login" {
                    return;
                } else {
                    let url = Origin::parse("/Token_UnAuthorized").unwrap();
                    req.set_uri(url);
                    return;
                }
            }
        }
        /*************************************************************************************
        如上代码用于验证token，并且是POST方法才生效*/
    }

    // async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
    //     println!("{:#?},{:#?}", response.status(), response.body());
    // }
}

//创建JWT结构体
#[derive(Serialize, Deserialize, Debug, Clone, FromForm)]
pub struct LoginUser {
    #[field(name=uncase("userphone"))]
    pub userPhone: String,
    #[field(name=uncase("smscode"))]
    pub smsCode: String,
    pub token: String,
}
//创建LoginResponse结构体
#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct LoginResponse {
    #[field(name=uncase("userphone"))]
    pub userPhone: String,
    #[field(name=uncase("smscode"))]
    pub smsCode: i32,
    pub token: String,
    pub code: i32, // 0：成功，非0：失败
    #[field(name=uncase("errmsg"))]
    pub errMsg: String,
}
impl LoginResponse {
    pub fn new(token: String, data: LoginUser, code: i32, errMsg: String) -> Self {
        LoginResponse {
            code,
            token,
            userPhone: data.userPhone,
            smsCode: 0,
            errMsg,
        }
    }
}
//创建FlowForm返回结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct FlowItemList {
    eventName: String,
    rn: i32,
    fstatus: String,
    fnumber: String,
    fformtype: String,
    fdisplayname: String,
    todostatus: i32,
    fname: String,
    senderphone: String,
    freceivernames: String,
    fphone: String,
    fprocinstid: String,
    fcreatetime: String,
}

//创建JWT结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}

impl Claims {
    pub fn new(sub: String) -> Self {
        let nowTimeStamp = jsonwebtoken::get_current_timestamp();
        let exp = nowTimeStamp + 31 * 24 * 60 * 60; //设置token过期时间为一个月
        Claims { sub, exp }
    }

    pub async fn get_token(usrPhone: String) -> String {
        let mut secretKey =
            env::var("TokenSecretKey").unwrap_or_else(|_| String::from("kephi520."));

        let mut hasherSecretKey = Sha256::new();
        hasherSecretKey.input_str(secretKey.as_ref());
        secretKey = hasherSecretKey.result_str();

        let claims = Claims::new(usrPhone.to_owned());

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secretKey.as_ref()),
        )
        .unwrap();
        // println!("token:{:#?}", token);
        token
    }
    pub async fn verify_token(token: String) -> bool {
        let mut secretKey =
            env::var("TokenSecretKey").unwrap_or_else(|_| String::from("kephi520."));

        let mut hasherSecretKey = Sha256::new();
        hasherSecretKey.input_str(secretKey.as_ref());
        secretKey = hasherSecretKey.result_str();

        let mut validate = Validation::new(Algorithm::HS256);
        validate.leeway = 0; //设置偏差为0

        let deToken = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secretKey.as_ref()),
            &validate,
        );
        //println!("{:#?}", deToken);
        match deToken {
            Ok(_) => {
                return true;
            }
            Err(_) => return false,
        }
    }
}
