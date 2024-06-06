//引入rocket
#[allow(unused)]
use rocket::{
    self, build, config::Config, fairing::AdHoc, get, http::Method, launch, post, routes, Shutdown,
};

//处理同源的问题
use rocket_cors::{AllowedOrigins, CorsOptions};

//标准库Result
pub use std::fmt;
use std::{net::{IpAddr, Ipv4Addr}, sync::{Arc,Mutex}};
pub use std::result::Result as std_Result;
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


lazy_static!{
    static ref IS_WORKING: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

#[rocket::main]
async fn main() -> std_Result<(), rocket::Error> {
    //初始化trancing日志追踪
    init();

    //创建一个测试循环来判断
    let _task = tokio::task::spawn(async move {
        loop {
            {
                let mut value = IS_WORKING.lock().unwrap();
                *value = true;               
            }
            println!("loop was running");
            tokio::time::sleep(std::time::Duration::from_secs(8)).await;
            println!("loop was stoped");            
            {
                let mut value = IS_WORKING.lock().unwrap();
                *value = false;               
            } 
            tokio::time::sleep(std::time::Duration::from_secs(16)).await;
            
        }
    });


    //创建消息对象用于生成数据库连接池
    let sendmsg = SendMSG::new();
    let pools = sendmsg.buildpools().unwrap();

    //消息接口处理方法的封装
    let _smg = local_thread().await;

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
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 8000,
        //cli_colors: false,
        ..Default::default()
    };

    //rocket服务器启动
    let _rocket = rocket::custom(config)
        //::build()
        .attach(cors)
        .manage(pools)
        .mount("/", routes![index, phone, shutdown])
        .launch()
        .await?;

    info!("程序结束");

    Ok(())
}
