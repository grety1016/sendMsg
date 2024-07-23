use std::borrow::Borrow;

//引入rocket
#[allow(unused)]
use rocket::{
    self, build, config::Config, fairing::AdHoc, get, http::Method, http::Status, launch, post,
    routes, serde::json::Json, Shutdown, State,
};
//引入serde_json
use serde_json::json;

//日志跟踪
pub use tracing::{event, info, trace, warn, Level};

//引入mssql
use mssql::*;
//引入全局变量
use crate::IS_WORKING;

//引入用户类型
pub mod interface;
pub use interface::*;

#[get("/?<phone>")]
pub async fn phone(phone: String, pools: &State<Pool>) -> String {
    let conn = pools.get().await.unwrap();
    let is_exist = conn
        .exec(sql_bind!(
            "IF EXISTS(SELECT 1 FROM UserID WITH (NOLOCK) WHERE userphone = @p1) select 1",
            &phone
        ))
        .await
        .unwrap();
    if is_exist == 0 {
        return "游客，您好！当前用户不存在,请联系管理员咨询！".to_string();
    }

    let result = conn
        .query_scalar_string(sql_bind!(
            "UPDATE dbo.UserID SET jointime = getdate() WHERE  userphone = @p1; 
        SELECT username FROM UserID WITH(NOLOCK) WHERE userphone = @p1;
        ",
            &phone
        ))
        .await
        .unwrap();
    let name = result.unwrap();
    format!("{} 女士/先生,欢迎您加入快先森金蝶消息接口！", name)
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

#[get("/")]
pub async fn index(pools: &State<Pool>) -> &'static str {
    let conn = pools.get().await.unwrap();

    let mut result = conn
        .query("SELECT top 1 1 FROM dbo.T_SEC_USER")
        .await
        .unwrap();
    if let Some(row) = result.fetch().await.unwrap() {
        println!("test is work:{:?}", row.try_get_i32(0).unwrap());
    }
    "您好,欢迎使用快先森金蝶消息接口!!!"
}

#[post("/login", format = "json", data = "<user>")]
pub async fn login(user: Json<LoginUser>, pools: &State<Pool>) -> Json<LoginResponse> {
    let Json(userp) = user;

    let conn = pools.get().await.unwrap();

    let login_user = conn
        .exec(sql_bind!(
            "SELECT  1  FROM dbo.sendMsg_users WHERE userName = @p1 AND userPwd = @p2",
            &userp.userName,
            &userp.userPwd
        ))
        .await
        .unwrap();

    if login_user == 0 {
        return Json(LoginResponse::new(userp.clone(), -1));
    } else {
        return Json(LoginResponse::new(userp.clone(), 0));
    }

    // 加入任务
}
