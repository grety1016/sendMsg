//tokio异步运行时
use tokio::time;
//reqwestHTTP请求
use httprequest::Client;
//hashmap
use std::collections::HashMap;
//系列化
use serde::{Serialize,Deserialize};



//钉钉获取token请求主体
#[derive(Debug,Serialize,Deserialize)]
struct DDToken {
    url:String,
    appkey: String,
    appsecret:String    
}
//DDTokenResult
#[derive(Debug,Serialize,Deserialize)]
struct DDTokenResult{
    errcode:u32,
    access_token:String,
    errmsg:String,
} 

//实现token请求主体
impl DDToken {
    //创建实例
    pub fn new () -> DDToken {
        DDToken { 
            //获取钉钉token的URL及参数
            url:String::from("https://oapi.dingtalk.com/gettoken"),
            appkey:"dingzblrl7qs6pkygqcn".to_string(), 
            appsecret:"26GGYRR_UD1VpHxDBYVixYvxbPGDBsY5lUB8DcRqpSgO4zZax427woZTmmODX4oU".to_string()
        }
    }

    //获取钉钉机器人token方法
    pub async fn get_token(&self) -> String {
        //将参数加入到一个hash变量
        let mut map = HashMap::new();
        map.insert("appkey", self.appkey.clone());
        map.insert("appsecret", self.appsecret.clone());        

        //新增一个客户端实例用来访问钉钉接口获取access_token
        let client = Client::new();
        let token_str = client.get(self.url.clone()).query(&map).send().await.unwrap().text().await.unwrap();
        let access_token:DDTokenResult =  serde_json::from_str(&token_str).unwrap();
        
        access_token.access_token     

    }    

}


#[tokio::main]
async fn main() {
    let token = DDToken::new();
    let access_token = token.get_token().await;
    println!("token:{}",access_token);

    
    
}
