//引入rocket
use rocket::{
    self, build, config::Config, fairing::AdHoc, get, http::Method, launch, post, routes, Shutdown,
};
#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/shutdown")]
pub fn shutdown(shutdown:Shutdown) -> &'static str{ 
    shutdown.notify();
    "优雅关机！"
}