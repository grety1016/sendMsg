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
        //将获取token参数加入到一个hash变量
        let mut get_token_param = HashMap::new();
        get_token_param.insert("appkey", self.appkey.clone());
        get_token_param.insert("appsecret", self.appsecret.clone());        

        //新增一个客户端实例用来访问钉钉接口获取access_token
        let client = Client::new();
        let token_str = client.get(self.url.clone()).query(&get_token_param).send().await.unwrap().text().await.unwrap();
        let access_token:DDTokenResult =  serde_json::from_str(&token_str).unwrap();
        
        access_token.access_token     

    }    

}

//通过useriphone获取userid
#[derive(Debug,Serialize,Deserialize)]
struct DDUserid {
    url: String,
    access_token: String,
    mobile: String,
}


//userid返回类型
#[derive(Debug,Serialize,Deserialize)]
struct DDUseridResult {
    errcode:u32,
    errmsg:String,
    result:DDUseridValue,    
    request_id:String,

}

#[derive(Debug,Serialize,Deserialize)]
struct DDUseridValue {
    userid:String,
}
//钉钉DDUserid实现
impl DDUserid {
    pub fn new(access_token: String, mobile:String) -> DDUserid {
        DDUserid{
            url:"https://oapi.dingtalk.com/topapi/v2/user/getbymobile".to_string(),
            access_token,
            mobile
        }
    }

    pub async fn get_userid(&self) -> String { 
        let mut access_token = HashMap::new();
        access_token.insert("access_token", self.access_token.clone());
        //let mut mobile = HashMap::new();
        access_token.insert("mobile", self.mobile.clone());
         

        //新增一个客户端实例用来访问钉钉接口获取userid
        let client = Client::new(); 
        let useridresult =  
        client.post("https://oapi.dingtalk.com/topapi/v2/user/getbymobile").query(&access_token).send().await.unwrap().text().await.unwrap(); 
        let userid:DDUseridResult = serde_json::from_str(&useridresult).unwrap();
        userid.result.userid 

    }
}

struct User {
    exeuser:String,
    flownumber:String,
    flowname:String,
    userphone:String,
}

impl User {
    //初始化用户实例
    pub fn new(exeuser:String, flownumber:String,flowname:String,userphone:String) -> User {
        User {exeuser,flownumber, flowname, userphone}
    }

    //获取用于访问钉钉机器人的token
    pub async fn get_token(&self) -> String {
        let dd_get_token = DDToken::new();
        let dd_access_token = dd_get_token.get_token().await;
        dd_access_token
    }

    //通过用户手机获取userid
    pub async fn get_userid(&self,dd_access_token:String,mobile:String) -> String {
        //通过手机获取userid
        let dd_get_userid = DDUserid::new(dd_access_token, mobile);
        let dd_userid = dd_get_userid.get_userid().await; 
        dd_userid 
    }
}
 




#[tokio::main]
async fn main() {
    let user = User::new("苏宁绿".to_string(),"EBS20240525000001_20240525135052".to_string(),"您有待办任务需要处理".to_string(),"15345923407".to_string());
    let access_token = user.get_token().await;
    let userid=user.get_userid(access_token, user.userphone.clone()).await;
    println!("{}",userid);
    

     

     
    
}
