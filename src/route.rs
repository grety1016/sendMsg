use std::{borrow::Borrow, io, result::Result};

use httprequest::Request;
use rocket::{data::{self, FromData}, http::hyper::request};
//引入rocket
#[allow(unused)]
use rocket::{
    data::{Data, ToByteUnit},
    self, build,
    config::Config,
    fairing::AdHoc,
    futures::{SinkExt, StreamExt},
    get,
    http::Method,
    http::Status,
    launch, post,
    request::{FromRequest, Outcome},
    routes,
    serde::json::Json,
    tokio::sync::broadcast::{Receiver, Sender},
    Request as rocketRequest, Shutdown, State,
};
//引入rocket_ws
use rocket_ws::{self, stream::DuplexStream, Message, WebSocket};
//引入tokio
use tokio::{self, select};
//引入serde_json
use serde::{de::value::CowStrDeserializer, Deserialize, Serialize};
use serde_json::json; //用于结构体上方的系列化宏

//日志跟踪
pub use tracing::{event, info, trace, warn, Level};

//引入mssql
use mssql::*;
//引入全局变量
use crate::IS_WORKING;

use either::*;

pub mod route_method;
use route_method::*;
//随机生成数字 
use rand::Rng;

 

 

#[get("/ws")]
pub async fn ws(
    ws: WebSocket,
    tx: &State<Sender<String>>
) -> rocket_ws::Channel<'static> {
    let mut rx = tx.subscribe();
    ws.channel(move |mut stream| {
        Box::pin(async move {
            let mut _stream_clone = &stream;
            loop {
                select! {
                    //等待接收前端消息来执行事件函数
                   Some(msg) = stream.next() =>{
                        match msg {
                            Ok(msg) => {
                                handle_message(&mut stream,msg).await?;
                            },
                            Err(e)=> info!("{}", e),
                        }
                   }
                   //后端事件执行后触发消息机制响应
                   msg = rx.recv() => {
                    match msg {
                    Ok(msg) => {
                        stream.send(msg.into()).await?;
                    },

                    Err(e)=> info!("{}", e),
                    }

                   }
                }
            }
        })
    })
}
//如下函数用于执行接收消息后的处理函数
async fn handle_message(
    stream: &mut DuplexStream,
    msg: Message,
) -> Result<(), rocket_ws::result::Error> {
    stream.send(msg.into()).await?;
    Ok(())
}

#[get("/getsmscode?<userphone>")]
pub async fn getSmsCode(userphone: String, pools: &State<Pool>) -> Json<LoginResponse> {
    let mut random_number:i32=0;
    let mut code = 0;
    let mut errMsg = "".to_owned(); 
    let mut smsCode = 0;

    let conn = pools.get().await.unwrap();

    let result = conn.query_scalar_i32(sql_bind!("SELECT  DATEDIFF(second, createdtime, GETDATE())  FROM dbo.sendMsg_users WHERE userPhone = @p1", &userphone)).await.unwrap();
    if let Some(val) = result{
        if val < 60 {
            code = -2;
            errMsg="操作过于频繁，请复制最近一次验证码或一分钟后重试".to_owned();
        }
        let mut rng = rand::thread_rng();
        random_number  = rng.gen_range(100000..1000000);     
    }else {
        errMsg = "该手机号未注册!".to_owned();
        code = -1;
    }
    if code == 0{
        smsCode = random_number.clone(); 
        let _exec = conn.exec(sql_bind!("UPDATE dbo.sendMsg_users SET smsCode = @p1,createdtime = getdate() WHERE userPhone = @p2",random_number,&userphone)).await.unwrap();  
    }
           
    
             
    return Json(LoginResponse{userPhone:userphone,smsCode,token:"".to_owned(),code,errMsg});
 
}

#[get("/shutdown")]
pub fn shutdown(shutdown: Shutdown) -> &'static str {
    let value = IS_WORKING.lock().unwrap();
    if *value {
        "任务正在执行中,请稍后重试！"
    } else {
        shutdown.notify();
        "优雅关机!!！"
    }
}

//接收文本消息结构中的文字
#[derive(Debug, Deserialize, Serialize)]
pub struct Text{
    pub content:String,
}
//接收消息文本结构 
#[derive(Debug, Deserialize, Serialize)]
pub struct RecvMessage{
    pub senderStaffId:String,
    pub text:Option<Text>,
    pub content:Option<Content>,
    pub msgtype:String,
} 
//接收语音消息结构中的文字
#[derive(Debug, Deserialize, Serialize)]
pub struct Content{
    pub recognition:String,
} 

 

#[post("/receiveMsg",format = "json", data = "<data>")]
pub async fn receiveMsg(data:Json<RecvMessage>) -> &'static str {
    println!("{:#?}", data);
    "post_index"
}

#[get("/")]
pub async fn index(pools: &State<Pool>) -> &'static str {
    let conn = pools.get().await.unwrap();

    let mut result = conn
        .query("SELECT top 1 1 FROM dbo.T_SEC_USER")
        .await
        .unwrap();
    if let Some(row) = result.fetch().await.unwrap() {
        println!("test is work:{:?}!", row.try_get_i32(0).unwrap());
    }
    "您好,欢迎使用快先森金蝶消息接口!!!"
}

#[post("/login", format = "json", data = "<user>")]
pub async fn login<'r>(user: Json<LoginUser>, _pools: &State<Pool>) -> Json<LoginResponse> {
    let Json(mut userp) = user;
    // assert_eq!(userp.token.is_empty(),false);
    // assert_eq!(Claims::verify_token(userp.token.clone()).await,true);
    if !userp.token.is_empty() && Claims::verify_token(userp.token.clone()).await {
        // println!("token验证成功：{:#?}", &userp.token);
        return Json(LoginResponse::new(
            userp.token.clone(),
            userp,
            0,
            "".to_string(),
        ));
    } else {
        if userp.userPhone.is_empty() || userp.smsCode.is_empty() {
            // println!("用户名或密码为空：{:#?}", &userp.token);
            return Json(LoginResponse::new(
                "Bearer".to_string(),
                userp.clone(),
                -1,
                "手机号或验证码不能为空!".to_string(),
            ));
        } else {
            // let conn = pools.get().await.unwrap();
            // let userPhone = conn
            //     .query_scalar_string(sql_bind!(
            //         "SELECT  userPhone  FROM dbo.sendMsg_users WHERE userName = @p1 AND userPwd = @p2",
            //         &userp.userName,
            //         &userp.userPwd
            //     ))
            //     .await
            //     .unwrap();
            let userPhone = Some("15345923407".to_owned());
            let mut token = String::from("Bearer");
            let code: i32;
            let mut errmsg = String::from("");

            if let Some(value) = userPhone {
                token = Claims::get_token(value.to_owned()).await;
                userp.smsCode="000000".to_owned();
                code = 0;
            } else {
                code = -1;
                errmsg = "用户名或密码错误!".to_owned();
            }
            // if code == 0 {println!("创建token成功：{:#?}", &userp.token);}else{println!("用户名或密码错误!")}
            return Json(LoginResponse::new(token, userp.clone(), code, errmsg));
        }
    } // 加入任务
}

// #[get("/unauthorized")]
// pub async fn unauthorized() -> &'static str {
//     println!("unauthorized");
//     return "unauthorized"
// }
#[post("/Token_UnAuthorized", format = "json", data = "<user>")]
pub async fn Token_UnAuthorized(user: Json<LoginUser>) -> Json<LoginResponse> {
    let Json(userp) = user;

    // println!("unauthorized");
    return Json(LoginResponse::new(
        "Bearer".to_string(),
        userp.clone(),
        -1,
        "Token_UnAuthorized".to_string(),
    ));
}
