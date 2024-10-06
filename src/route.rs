use httprequest::Request;
use std::{borrow::Borrow, io, result::Result, sync::Arc, time::Duration};
use tracing::field;
//引入rocket
use rocket::{
    self, build,
    config::Config,
    data::{self, FromData},
    data::{Data, ToByteUnit},
    fairing::AdHoc,
    form::{self, Form},
    fs::relative,
    fs::TempFile,
    futures::{SinkExt, StreamExt},
    get,
    http::hyper::request,
    http::Status,
    launch, post,
    response::{
        status,
        stream::{Event, EventStream},
    },
    routes,
    serde::json::Json,
    tokio::sync::broadcast::{Receiver, Sender},
    FromForm, Request as rocketRequest, Response, Shutdown, State,
};
//引入rocket_ws
use rocket_ws::{self, stream::DuplexStream, Message, WebSocket};
//引入tokio
use tokio::{self, select, task, time};
//引入serde_json
use serde::{de::value::CowStrDeserializer, Deserialize, Serialize};
use serde_json::json; //用于结构体上方的系列化宏

//日志跟踪
pub use tracing::{event, info, trace, warn, Level};

//引入mssql
use mssql::*;
//引入seq-obj-id
use seqid::*;
//引入全局变量
use crate::IS_WORKING;

use either::*;

pub mod route_method;
use route_method::*;
//随机生成数字
use rand::Rng;

//引入sendmsg模块

use crate::sendmsg::*;

#[derive(FromForm, Debug)]
pub struct Upload<'r> {
    files: Vec<TempFile<'r>>,
}
#[post("/upload", format = "multipart/form-data", data = "<form>")]
pub async fn upload(mut form: Form<Upload<'_>>) {
    // let result = form.files.persist_to("D:/public/trf.txt").await;
    // println!("{:#?}",result);

    for file in form.files.iter_mut() {
        println!("file's name:{:#?}", file.name());
        println!(
            "file's name:{:#?}",
            file.content_type().unwrap().to_string()
        );
    }
}
//websocket connection
#[get("/ws")]
pub async fn ws(ws: WebSocket, tx: &State<Sender<String>>) -> rocket_ws::Channel<'static> {
    let mut rx = tx.subscribe();
    ws.channel(move |mut stream| {
        Box::pin(async move {
            // let mut _stream_clone = &stream;
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
    stream.send(msg).await?;
    Ok(())
}

//SSE 连接
#[get("/event_conn")]
pub async fn event_conn() -> EventStream![] {
    println!("event_conn");
    let mut num = 0;
    EventStream! {
        loop{
            time::sleep(Duration::from_secs(1)).await;
            num+=1;
            yield Event::data(format!("form server message{}",num));
        }
    }
}

#[get("/getsmscode?<userphone>")]
pub async fn getSmsCode(userphone: String, pools: &State<Pool>) -> Json<LoginResponse> {
    let mut random_number: i32 = 0;
    let mut code = 0;
    let mut errMsg = "".to_owned();
    let mut smsCode = 0;

    let conn = pools.get().await.unwrap();
    //查询当前手机是否在消息用户列表中存在有效验证码
    let result = conn.query_scalar_i32(sql_bind!("SELECT  DATEDIFF(second, createdtime, GETDATE())  FROM dbo.sendMsg_users WHERE userPhone = @p1", &userphone)).await.unwrap();
    //存在后判断最近一次发送时长是否在60秒内
    if let Some(val) = result {
        if val <= 60 {
            code = -2;
            errMsg = "操作过于频繁，请复制最近一次验证码或一分钟后重试".to_owned();
        } else {
            let mut rng = rand::thread_rng();
            random_number = rng.gen_range(1000..10000);
        }
    } else {
        errMsg = "该手机号未注册!".to_owned();
        code = -1;
    }
    //如果用户存在并在60秒内未发送验证码，则发送验证码
    if code == 0 {
        smsCode = random_number;
        let sendinfo = conn.query_first_row(sql_bind!("UPDATE dbo.sendMsg_users SET smsCode = @p1,createdtime = getdate() WHERE userPhone = @p2
        SELECT  dduserid,userphone,robotcode,smscode   FROM sendMsg_users  WITH(NOLOCK)  WHERE userphone = @P2
        ",random_number,&userphone)).await.unwrap().unwrap();

        let mut smscode = SmsMessage::new(
            "".to_owned(),
            sendinfo.try_get_str(0).unwrap().unwrap(),
            sendinfo.try_get_str(1).unwrap().unwrap(),
            sendinfo.try_get_str(2).unwrap().unwrap(),
            sendinfo.try_get_i32(3).unwrap().unwrap(),
        );

        if smscode.get_rotobotcode() == "dingrw2omtorwpetxqop" {
            let gzym_ddtoken = DDToken::new(
                "https://oapi.dingtalk.com/gettoken",
                "dingrw2omtorwpetxqop",
                "Bcrn5u6p5pQg7RvLDuCP71VjIF4ZxuEBEO6kMiwZMKXXZ5AxQl_I_9iJD0u4EQ-N",
            );
            smscode.set_ddtoken(gzym_ddtoken.get_token().await);
        } else {
            let zb_ddtoken = DDToken::new(
                "https://oapi.dingtalk.com/gettoken",
                "dingzblrl7qs6pkygqcn",
                "26GGYRR_UD1VpHxDBYVixYvxbPGDBsY5lUB8DcRqpSgO4zZax427woZTmmODX4oU",
            );
            smscode.set_ddtoken(zb_ddtoken.get_token().await);
        }
        smscode.send_smsCode().await;
    }

    Json(LoginResponse {
        userPhone: userphone,
        smsCode,
        token: "".to_owned(),
        code,
        errMsg,
    })
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
pub struct Text {
    pub content: String,
}
//接收消息文本结构
#[derive(Debug, Deserialize, Serialize)]
pub struct RecvMessage {
    pub senderStaffId: String,
    pub text: Option<Text>,
    pub content: Option<Content>,
    pub msgtype: String,
}
//接收语音消息结构中的文字
#[derive(Debug, Deserialize, Serialize)]
pub struct Content {
    pub recognition: String,
}

#[post("/receiveMsg", format = "json", data = "<data>")]
pub async fn receiveMsg(data: Json<RecvMessage>) {
    println!("{:#?}", data);
}

#[get("/test")]
pub async fn test_fn(_pools: &State<Pool>) -> Result<Json<Content>, String> {
    // Ok(Json(Content{recognition:"Ok".into()}))
    Err("test_ERROR".into())
}

#[get("/")]
pub async fn index(pools: &State<Pool>) -> status::Custom<&'static str> {
    let conn = pools.get().await.unwrap();

    let mut result = conn
        .query("SELECT top 1 1 FROM dbo.T_SEC_USER")
        .await
        .unwrap();
    if let Some(row) = result.fetch().await.unwrap() {
        println!("server is working:{:?}!", row.try_get_i32(0).unwrap());
    }
    crate::local_thread().await;
    status::Custom(Status::Ok, "您好,欢迎使用快先森金蝶消息接口!!!")
}

#[post("/login", format = "json", data = "<user>")]
pub async fn login<'r>(user: Json<LoginUser>, pools: &State<Pool>) -> Json<LoginResponse> {
    let Json(userp) = user;
    // assert_eq!(userp.token.is_empty(),false);
    // assert_eq!(Claims::verify_token(userp.token.clone()).await,true);
    if !userp.token.is_empty() && Claims::verify_token(userp.token.clone()).await {
        // println!("token验证成功：{:#?}", &userp.token);
        Json(LoginResponse::new(
            userp.token.clone(),
            userp,
            0,
            "".to_string(),
        ))
    } else if userp.userPhone.is_empty() || userp.smsCode.is_empty() {
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
    // 加入任务
}

#[get("/getitemlist?<userphone>&<itemstatus>")]
pub async fn getItemList(
    userphone: String,
    itemstatus: String,
    pool: &State<Pool>,
) -> Json<Vec<FlowItemList>> {
    let conn = pool.get().await.unwrap();
    println!("userphone:{},itemstatus:{}", &userphone, &itemstatus);
    let flowitemlist: Vec<FlowItemList> = conn
        .query_collect(sql_bind!(
            "SELECT * FROM getTodoList(@p1,@p2)",
            &itemstatus,
            &userphone
        ))
        .await
        .unwrap();
    Json(flowitemlist)
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
    Json(LoginResponse::new(
        "Bearer".to_string(),
        userp,
        -1,
        "Token_UnAuthorized".to_string(),
    ))
}
