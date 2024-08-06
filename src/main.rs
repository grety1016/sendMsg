#![allow(non_snake_case)]
#![allow(unused_imports)]

//引入rocket
#[allow(unused)]
use rocket::{
    self, build,
    config::Config,
    fairing::AdHoc,
    futures::{SinkExt, StreamExt},
    get,
    http::Method,
    launch, post, routes,
    tokio::{
        self,
        sync::{broadcast, mpsc},
        task::spawn,
        time::{sleep, Duration},
    },
    Shutdown,
};

//处理同源的问题
use rocket_cors::{AllowedOrigins, CorsOptions};
use route_method::TokenFairing;

//标准库Result
pub use std::fmt;
pub use std::result::Result as std_Result;
#[allow(unused)]
use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, Mutex},
};
//消息接口模块
pub mod sendmsg;
use sendmsg::*;

//路由定义模块
pub mod route;
use route::*;

//日志追踪模块
pub mod log_record;
pub use log_record::*;

//使用静态库
use lazy_static::lazy_static;

lazy_static! {
    static ref IS_WORKING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[rocket::main]
async fn main() -> std_Result<(), rocket::Error> {
    //初始化trancing日志追踪
    init();

    //创建一个测试循环来判断
    // let _task = tokio::task::spawn(async move {
    //     loop {
    //         {
    //             let mut value = IS_WORKING.lock().unwrap();
    //             *value = true;
    //         }
    //         //消息接口处理方法的封装
    //         let _smg = local_thread().await;
    //         info!("Task is working!");
    //         tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    //         {
    //             let mut value = IS_WORKING.lock().unwrap();
    //             *value = false;
    //         }
    //         info!("Task was done!");
    //         tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    //     }
    // });

    //创建消息对象用于生成数据库连接池
    let sendmsg = SendMSG::new();
    let pools = sendmsg.buildpools(60, 8).unwrap();

    //创建多播消息通道
    #[allow(unused)]
    let (tx, mut rx) = broadcast::channel::<String>(200);

    let tx2 = tx.clone();
    // tokio::spawn(async move {
    //     loop {
    //         tokio::time::sleep(Duration::from_millis(3000)).await;
    //         tx2.send(String::from("Task1 completed!")).unwrap();
    //     }
    // });
    // let tx3 = tx.clone();

    // tokio::spawn(async move {
    //     loop {
    //         tokio::time::sleep(Duration::from_millis(3000)).await;
    //         tx3.send(String::from("Task2 completed!")).unwrap();
    //     }
    // });

    //使用rocket_cors处理跨域同源策略问题：
    let allowed_origins = AllowedOrigins::all();
    //cors请求处理配置
    let cors = CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post, Method::Put, Method::Delete]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("CORS configuration failed");

    //rocket启动配置
    let config = Config {
        //tls: Some(tls_config),需要增加TLS时使用
        address: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 31)),
        // address: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 31)),
        port: 80,
        //cli_colors: false,
        ..Default::default()
    };

    //rocket服务器启动
    let _rocket = rocket::custom(config)
        //::build()
        .attach(TokenFairing)
        .attach(cors)
        .manage(pools)
        .manage(tx)
        .manage(rx)
        .mount("/", routes![index, phone, shutdown, Token_UnAuthorized, ws])
        .mount("/user", routes![login])
        .launch()
        .await?;

    info!("程序结束");

    Ok(())
}
