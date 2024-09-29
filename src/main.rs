#![allow(non_snake_case)]
#![allow(unused_imports)]

use rocket::figment::value::magic::RelativePathBuf;
//引入rocket
#[allow(unused)]
use rocket::{
    self, build,
    config::Config,
    data::{Limits, ToByteUnit},
    fairing::AdHoc,
    figment::Figment,
    fs::{relative, FileServer},
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
pub use std::result::Result as std_Result;
pub use std::{fmt, process, thread};
#[allow(unused)]
use std::{
    fs::File,
    net::{IpAddr, Ipv4Addr, UdpSocket},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::task;

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

//配置静态文件

#[rocket::main]
async fn main() -> std_Result<(), rocket::Error> {
    // 为了确保所有线程完成，主线程需要等待一段时间
    thread::sleep(Duration::from_secs(10));

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
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), //外网地址： http://8sjqkmbn.beesnat.com/
        // address: IpAddr::V4(Ipv4Addr::new(192, 168, 0, 31)),
        port: 8000,
        temp_dir: RelativePathBuf::from("/2、RustProgramming/sendMsg/temp"),
        //cli_colors: false,
        ..Default::default()
    };

    let limits = Limits::new()
        .limit("json", 20.mebibytes())
        .limit("file", 20.mebibytes())
        .limit("file/zip", 20.mebibytes())
        .limit("bytes", 20.mebibytes())
        .limit("form-data", 20.mebibytes())
        .limit("data-form", 20.mebibytes())
        .limit("form", 20.mebibytes());

    //rocket启动配置合并文件大小限制
    let figment = Figment::from(config).merge(("limits", limits));
    //用于合并加载上方limits文件限制配置，但启用这个配置会有一个问题：当文件超过限制时会提示413状态码，无法直接处理返回异常信息,这边尽可能设置更大的限制以允许文件上传，在处理器中做大小限制
    //.merge(("limits", limits));

    //rocket服务器启动
    let _rocket = rocket::custom(figment)
        //::build()
        .attach(TokenFairing)
        // .attach(cors)
        .manage(pools)
        .manage(tx)
        .manage(rx)
        .mount("/public", FileServer::from("D:/public"))
        .mount(
            "/",
            routes![
                index,
                Token_UnAuthorized,
                receiveMsg,
                upload,
                test_fn,
                event_conn
            ],
        )
        .mount("/user", routes![login, getSmsCode])
        .mount("/flowform", routes![getItemList])
        .launch()
        .await?;

    info!("程序结束");

    Ok(())
}
