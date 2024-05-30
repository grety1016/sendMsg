//tokio异步运行时
use tokio::time;
//reqwestHTTP请求
use httprequest::Client;
//hashmap
use std::{collections::HashMap, result};
//系列化
use serde::{Deserialize, Serialize};

//引入数据库
use mssql::Pool;
use mssql::*;

//钉钉获取token请求主体
#[derive(Debug, Serialize, Deserialize)]
struct DDToken<'r> {
    url: &'r str,
    appkey: &'r str,
    appsecret: &'r str,
}
//DDTokenResult
#[derive(Debug, Serialize, Deserialize)]
struct DDTokenResult<'r> {
    errcode: u32,
    access_token: &'r str,
    errmsg: &'r str,
}

//实现token请求主体
impl<'r> DDToken<'r> {
    //创建实例
    pub fn new(url: &'r str, appkey: &'r str, appsecret: &'r str) -> DDToken<'r> {
        DDToken {
            //获取钉钉token的URL及参数
            url,
            appkey,
            appsecret,
        }
    }

    //获取钉钉机器人token方法
    pub async fn get_token(&self) -> String {
        //将获取token参数加入到一个hash变量
        let mut get_token_param = HashMap::new();
        get_token_param.insert("appkey", self.appkey);
        get_token_param.insert("appsecret", self.appsecret);

        //新增一个客户端实例用来访问钉钉接口获取access_token
        let client = Client::new();
        let token_str = client
            .get(self.url)
            .query(&get_token_param)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let access_token: DDTokenResult = serde_json::from_str(&token_str).unwrap();
        //println!("{:#?}", access_token);
        access_token.access_token.to_owned()
    }
}

//通过useriphone获取userid
#[derive(Debug, Serialize, Deserialize)]
struct DDUserid;
//userid返回类型
#[derive(Debug, Serialize, Deserialize)]
struct DDUseridResult<'r> {
    errcode: u32,
    result: DDUseridValue<'r>,
    errmsg: &'r str,
    request_id: &'r str,
}

impl<'r> DDUseridResult<'r> {
    pub fn new() -> DDUseridResult<'r> {
        DDUseridResult {
            errcode: 0,
            result: DDUseridValue { userid: "" },
            errmsg: "",
            request_id: "",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DDUseridValue<'r> {
    userid: &'r str,
}
//钉钉DDUserid实现
impl DDUserid {
    pub fn new() -> DDUserid {
        DDUserid
    }

    pub async fn get_userid<'r>(&self,access_token: &'r str, mobile: &'r str) -> String {
        let mut request: HashMap<String, String> = HashMap::new();
        request.insert("access_token".to_owned(), access_token.to_owned());
        //let mut mobile = HashMap::new();
        request.insert("mobile".to_owned(), mobile.to_owned());

        //新增一个客户端实例用来访问钉钉接口获取userid
        let client = Client::new();
        let useridresult = client
            .post("https://oapi.dingtalk.com/topapi/v2/user/getbymobile")
            .query(&request)
            .send()
            .await
            .unwrap()
            .text()
            .await;
        //println!("{:#?}", useridresult);
        let mut userid = DDUseridResult::new();
        let mut x = String::new();
        match useridresult {
            Ok(v) => {
                x = v.clone();
                userid = serde_json::from_str(&x).unwrap();
            }
            Err(e) => println!("ErrMSG:{:#?}", e),
        }
        userid.result.userid.to_owned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct User<'r> {
    exeuser: &'r str,
    flownumber: &'r str,
    access_token: Option<&'r str>,
    userphone: &'r str,
    userid: Option<&'r str>,
    robotcode: &'r str,
    msgkey: &'r str,
    msgparams: &'r str,
}

impl<'r> User<'r> {
    //初始化用户实例,用于测试调试用
    pub fn new(
        exeuser: &'r str,
        flownumber: &'r str,
        access_token: Option<&'r str>,
        userphone: &'r str,
        userid: Option<&'r str>,
        robotcode: &'r str,
        msgkey: &'r str,
        msgparams: &'r str,
    ) -> User<'r> {
        User {
            exeuser,
            flownumber,
            access_token,
            userphone,
            userid,
            robotcode,
            msgkey,
            msgparams,
        }
    }

    //发送消息到当前用户钉钉账号

    pub async fn send_msg(&mut self) {
        //创建请求表头结构
        let mut request_heads: Vec<String> = Vec::new();
        request_heads.push("x-acs-dingtalk-access-token".to_owned());
        request_heads.push(self.access_token.as_ref().unwrap().to_string());

        //创建请求表体结构
        let mut request_body = HashMap::new();
        request_body.insert("msgParam", self.msgparams);
        request_body.insert("msgKey", self.msgkey);
        request_body.insert("robotCode", self.robotcode);
        request_body.insert("userIds", self.userid.as_ref().unwrap());

        //HashMap转换成Json对象
        let request_body = serde_json::json!(request_body);

        //发起消息调用接口请求
        let client = Client::new();
        let _sendmsg = client
            .post("https://api.dingtalk.com/v1.0/robot/oToMessages/batchSend")
            .header(request_heads[0].clone(), request_heads[1].clone())
            .json(&request_body)
            .send()
            .await
            .unwrap()
            .text()
            .await;
        //println!("{}",sendmsg.unwrap());
    }
}

#[derive(Debug, Serialize)]
struct Userid<'r> {
    userphone: &'r str,
    userid: Option<String>,
}

impl<'r> Userid<'r> {
    pub fn new(userphone: &'r str) -> Self {
        Userid {
            userphone,
            userid: None,
        }
    }
}

//用来实现消息发送时调用的方法
struct SendMSG;

impl SendMSG {
    pub fn new() -> SendMSG {
        SendMSG
    }
    //返回数据库连接配置
    pub fn conn_str(&self) -> String {
        let host = "localhost";
        let database = "ZSKAIS20240101213214";
        let user = "sa";
        let pwd = "kephi";
        format!(
            r#"Server={};Database={};Uid={};Pwd="{}";TrustServerCertificate=true;"#, //integratedsecurity=sspi 用于进行本地用户验证，不需要user,pwd
            host, database, user, pwd
        )
    }
    //返回数据库连接池
    pub fn buildpools(&self) -> Result<Pool> {
        let pools = mssql::Pool::builder()
            .max_size(16)
            .idle_timeout(30 * 60)
            .min_idle(4)
            .max_lifetime(60 * 60 * 2)
            .build(&self.conn_str())
            .unwrap();
        Ok(pools)
    }
    //返回需要发送消息的行数
    pub async fn get_send_num(&self, pools: &Pool) -> i32 {
        let conn = pools.get().await.unwrap();

        let mut num: Option<i32> = Some(0);

        let _num2 = conn
            .scoped_trans(async {
                num = conn
                    .query_scalar_i32(
                        "
            DECLARE @num INT
            EXEC @num= get_flow_list
            SELECT @num",
                    )
                    .await
                    .unwrap();
                Ok(())
            })
            .await
            .unwrap();
        num.unwrap()
    }

    //获取userid表中未有userid的用户并回写useid
    pub async fn get_userid_list<'r>(&self, pools: &Pool, access_token:&'r str) {
        let mut userid_list: Vec<Userid> = Vec::new();
        let conn = pools.get().await.unwrap();

        let result: Vec<Row> = conn
            .query_collect_row(sql_format!(
                "SELECT trim(userphone),userid FROM {} where isnull(userid,'')=''",
                sql_ident!("UserID")
            ))
            .await
            .unwrap();

        for userid in result.iter() {
            userid_list.push(Userid::new(userid.try_get_str(0).unwrap().unwrap()));
            //println!("{:#?},{:#?}",userid.try_get_str(0).unwrap().unwrap(),userid.try_get_str(1).unwrap());
        }

        println!("access_token:{}",access_token);
        //初始化获取userid的对象
         let dduserid = DDUserid::new(); 
         let userid = dduserid.get_userid(access_token, "15345923407").await;
         println!("userid:{}",userid);
        

        for userid in userid_list.iter_mut() {
            let userid = Some(dduserid.get_userid(access_token, userid.userphone.as_ref()).await);
            println!("userid:{}",userid.unwrap());
        }
         
    }
    //执行消息发送
    pub async fn execute_send_msgs() {}
}

#[tokio::main]
async fn main() {
    let sendmsg = SendMSG::new();
    //获取一个数据连接池对象
    let pools = sendmsg.buildpools().unwrap();
    //获取数据库中满足发送消息的流程数量
    let sendmsgnum = sendmsg.get_send_num(&pools).await;
    println!("获取到需发送的列表用户数：{}", sendmsgnum);
    //初始化获取access_token的对象
    let ddtoken = DDToken::new(
        "https://oapi.dingtalk.com/gettoken",
        "dingrw2omtorwpetxqop",
        "Bcrn5u6p5pQg7RvLDuCP71VjIF4ZxuEBEO6kMiwZMKXXZ5AxQl_I_9iJD0u4EQ-N",
    );
    //获取实时access_token
    let access_token = ddtoken.get_token().await;
    //println!("access_token: {}", access_token);

    

    //获取userid列表回写表里
    let _userid_list = sendmsg.get_userid_list(&pools,access_token.as_str()).await;
}
