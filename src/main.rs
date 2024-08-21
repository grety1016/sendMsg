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
    net::{IpAddr, Ipv4Addr,UdpSocket},
    sync::{Arc, Mutex},
};
//消息接口模块
mod sendmsg;
use sendmsg::*;

//路由定义模块
pub mod route;
use route::*;

//日志追踪模块
pub mod log_record;
pub use log_record::*;

//网络请求
 
use httprequest::Client;

//MAC地址
use mac_address::get_mac_address;


//使用静态库
use lazy_static::lazy_static;

lazy_static! {
    static ref IS_WORKING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[rocket::main]
async fn main() -> std_Result<(), rocket::Error> {
    //初始化trancing日志追踪
    init();
    

    //创建消息对象用于生成数据库连接池
    let sendmsg = SendMSG::new();
    let pools = sendmsg.buildpools(60, 8).unwrap();

    //创建多播消息通道
    #[allow(unused)]
    let (tx, mut rx) = broadcast::channel::<String>(200);

   

    // //使用rocket_cors处理跨域同源策略问题：
    // let allowed_origins = AllowedOrigins::all();
    // //cors请求处理配置
    // let cors = CorsOptions {
    //     allowed_origins,
    //     allowed_methods: vec![Method::Get, Method::Post, Method::Put, Method::Delete]
    //         .into_iter()
    //         .map(From::from)
    //         .collect(),
    //     allowed_headers: rocket_cors::AllowedHeaders::all(),
    //     allow_credentials: true,
    //     ..Default::default()
    // }
    // .to_cors()
    // .expect("CORS configuration failed");

    //rocket启动配置
    let config = Config {
        //tls: Some(tls_config),需要增加TLS时使用
        address: IpAddr::V4(Ipv4Addr::new(192,168,0,31)),
        // address: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 31)),
        port: 8000,
        //cli_colors: false,
        ..Default::default()
    };

    //rocket服务器启动
    let _rocket = rocket::custom(config)
        //::build()
        .attach(TokenFairing)
        // .attach(cors)
        .manage(pools)
        .manage(tx)
        .manage(rx)
        .mount("/", routes![index,Token_UnAuthorized,receiveMsg])
        .mount("/user", routes![login,getSmsCode])
        .launch()
        .await?;

    info!("程序结束");

    Ok(())
}
