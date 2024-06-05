
//引入rocket
use rocket::{
    self, build, config::Config, fairing::AdHoc, get, http::Method, launch, post, routes, Shutdown, State,
};
//引入mssql
use mssql::*;

#[get("/?<name>&<phone>")]
pub async fn index(name:String,phone:String,pools:&State<Pool>) -> String {
    let conn = pools.get().await.unwrap();
    let result = conn.exec(sql_bind!("UPDATE dbo.UserID SET jointime = getdate() WHERE  userphone = @p1",phone)).await.unwrap();
    format!("{},欢迎您加入快先森金蝶消息接口！",name)
}

#[get("/shutdown")]
pub fn shutdown(shutdown:Shutdown) -> &'static str{ 
    shutdown.notify();
    "优雅关机！"
}