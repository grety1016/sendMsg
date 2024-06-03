//引入rocket
use rocket::{
    self, build, config::Config, fairing::AdHoc, get, http::Method, launch, post, routes, Shutdown,
};

//处理同源的问题
use rocket_cors::{AllowedOrigins, CorsOptions};

//标准库Result
pub use std::fmt;
pub use std::result::Result as std_Result;
//消息接口模块
pub mod smg;
use smg::*;

//路由定义模块
pub mod route;
use route::*;

//日志追踪模块
pub mod log_record;
pub use log_record::*;


#[rocket::main]
async fn main() -> std_Result<(), rocket::Error> {
    //初始化trancing日志追踪
    init();

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
        port: 8000,
        //cli_colors: false,

        ..Default::default()
    };

    //rocket服务器启动
    let _rocket = rocket::custom(config)
        //::build()
        .attach(cors)
        .mount("/", routes![index,shutdown])
        .launch()
        .await?;
    info!("This is an info log");
    event!(Level::ERROR, "This is an error log");
    trace!("This is an trance log");
    warn!("This is an warn log");
    Ok(())
}
