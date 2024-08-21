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

//引入sendmsg模块

use crate::sendmsg::*;
 

 

 

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
    //查询当前手机是否在消息用户列表中存在有效验证码
    let result = conn.query_scalar_i32(sql_bind!("SELECT  DATEDIFF(second, createdtime, GETDATE())  FROM dbo.sendMsg_users WHERE userPhone = @p1", &userphone)).await.unwrap();
    //存在后判断最近一次发送时长是否在60秒内
    if let Some(val) = result{
        if val <= 60 {
            code = -2;
            errMsg="操作过于频繁，请复制最近一次验证码或一分钟后重试".to_owned();
        }else{
            let mut rng = rand::thread_rng();
            random_number  = rng.gen_range(100000..1000000);             
        }            
    }else {
        errMsg = "该手机号未注册!".to_owned();
        code = -1;
    }
    //如果用户存在并在60秒内未发送验证码，则发送验证码
    if code == 0{
        smsCode = random_number.clone(); 
        let sendinfo = conn.query_first_row(sql_bind!("UPDATE dbo.sendMsg_users SET smsCode = @p1,createdtime = getdate() WHERE userPhone = @p2
        SELECT  dduserid,userphone,robotcode,smscode   FROM sendMsg_users  WITH(NOLOCK)  WHERE userphone = @P2
        ",random_number,&userphone)).await.unwrap().unwrap();
         
        let mut smscode = SmsMessage::new("".to_owned(), sendinfo.try_get_str(0).unwrap().unwrap(), sendinfo.try_get_str(1).unwrap().unwrap(), sendinfo.try_get_str(2).unwrap().unwrap(), sendinfo.try_get_i32(3).unwrap().unwrap());
        

        if smscode.robotcode == "dingrw2omtorwpetxqop"{
            let gzym_ddtoken = DDToken::new(
                "https://oapi.dingtalk.com/gettoken",
                "dingrw2omtorwpetxqop",
                "Bcrn5u6p5pQg7RvLDuCP71VjIF4ZxuEBEO6kMiwZMKXXZ5AxQl_I_9iJD0u4EQ-N",
            );
            smscode.ddtoken = gzym_ddtoken.get_token().await;
        }else {
            let zb_ddtoken = DDToken::new(
                "https://oapi.dingtalk.com/gettoken",
                "dingzblrl7qs6pkygqcn",
                "26GGYRR_UD1VpHxDBYVixYvxbPGDBsY5lUB8DcRqpSgO4zZax427woZTmmODX4oU",
            );
            smscode.ddtoken = zb_ddtoken.get_token().await;
            
        }
        smscode.send_smsCode().await;
        
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
        println!("server is working:{:?}!", row.try_get_i32(0).unwrap());
    }
    crate::local_thread().await;
    "您好,欢迎使用快先森金蝶消息接口!!!"
}

#[post("/login", format = "json", data = "<user>")]
pub async fn login<'r>(user: Json<LoginUser>, pools: &State<Pool>) -> Json<LoginResponse> {
    let Json(userp) = user;
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
            let conn = pools.get().await.unwrap();
            //查询当前用户列表中是否存在该手机及验证码，并且在3分钟时效内
            let userPhone = conn
                .query_scalar_string(sql_bind!(
                    "SELECT  userPhone  FROM dbo.sendMsg_users WHERE userphone = @P1 AND smscode = @P2 AND   DATEDIFF(MINUTE, createdtime, GETDATE()) <= 3",
                    &userp.userPhone,
                    &userp.smsCode
                ))
                .await
                .unwrap();
            let mut token = String::from("");
            // #[allow(unused)]
            let mut code: i32 = 0;
            let mut errmsg = String::from("");

            if let Some(value) = userPhone {
                token = Claims::get_token(value.to_owned()).await;
            } else {
                code = -1;
                errmsg = "手机号或验证码错误!".to_owned();
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
