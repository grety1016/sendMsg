#[allow(unused)]
use tokio::time;
//reqwestHTTP请求
use httprequest::Client;
#[allow(unused)]
use std::fmt::format;
//hashmap
#[allow(unused)]
use std::{collections::HashMap, result};
//系列化
use serde::{Deserialize, Serialize}; //用于结构体上方的系列化宏

//日志追踪
pub use tracing::{event, info, trace, warn, Level};

//引入数据库
pub use mssql::Pool;
pub use mssql::*;

//钉钉获取token请求主体
#[derive(Debug, Serialize, Deserialize)]
pub struct DDToken<'r> {
    url: &'r str,
    appkey: &'r str,
    appsecret: &'r str,
}
//获取token后的结果类型
#[derive(Debug, Serialize, Deserialize)]
pub struct DDTokenResult<'r> {
    errcode: u32,
    access_token: &'r str,
    errmsg: &'r str,
}

//实现token请求主体
impl<'r> DDToken<'r> {
    //创建实例(isworking)
    pub fn new(url: &'r str, appkey: &'r str, appsecret: &'r str) -> DDToken<'r> {
        DDToken {
            //获取钉钉token的URL及参数
            url,
            appkey,
            appsecret,
        }
    }

    //获取钉钉机器人token方法(isworking)
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

        access_token.access_token.to_owned()
    }
}

//通过useriphone获取userid
#[derive(Debug, Serialize, Deserialize)]
struct DDUserid;
//获取userid结果返回类型（包含错误类型）
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

//获取userid成功时usrid类型
#[derive(Debug, Serialize, Deserialize)]
struct DDUseridValue<'r> {
    userid: &'r str,
}
//钉钉DDUserid实现(isworking)
impl DDUserid {
    pub fn new() -> DDUserid {
        DDUserid
    }

    //调用通过手机获取userid(isworking)
    pub async fn get_userid<'r>(&self, access_token: &'r str, mobile: &'r str) -> String {
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
        info!("{:?}", useridresult);
        //新增一个获取ID返回结果类型
        let mut userid = DDUseridResult::new();
        #[allow(unused)]
        let mut x = String::new();

        //匹配返回结果，调用成功把值给userid
        match useridresult {
            Ok(v) => {
                x = v.clone();
                //匹配系列化结果，如果结果是可以转换的，转换成json
                match serde_json::from_str(&x) {
                    Ok(v) => userid = v,
                    Err(e) => event!(Level::ERROR, "turn_userid_error: {:?}", e),
                }
            }
            Err(e) => event!(Level::ERROR, "get_userid_error:{:?}", e),
        }
        userid.result.userid.to_owned()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsMessage<'r> {
    ddtoken: String,
    dduserid: &'r str,
    userphone: &'r str,
    robotcode: &'r str,
    smscode: i32,
}

impl<'r> SmsMessage<'r> {
    pub fn new(
        ddtoken: String,
        dduserid: &'r str,
        userphone: &'r str,
        robotcode: &'r str,
        smscode: i32,
    ) -> SmsMessage<'r> {
        SmsMessage {
            ddtoken,
            dduserid,
            userphone,
            robotcode,
            smscode,
        }
    }

    pub fn set_ddtoken(&mut self, ddtoken: String) {
        self.ddtoken = ddtoken;
    }

    pub fn get_rotobotcode(&self) -> &str {
        self.robotcode
    }

    pub async fn send_smsCode(&mut self) {
        //创建请求表头结构
        let mut request_heads: Vec<String> = Vec::new();
        request_heads.push("x-acs-dingtalk-access-token".to_owned());
        request_heads.push(self.ddtoken.to_string());
        let msgParams = format!(r#"{{ "msgtype": "text","content": "{}"}}"#, self.smscode);
        //创建请求表体结构
        let mut request_body = HashMap::new();
        request_body.insert("msgParam", msgParams);
        request_body.insert("msgKey", "sampleText".to_owned());
        request_body.insert("robotCode", self.robotcode.to_owned().clone());
        request_body.insert("userIds", self.dduserid.to_owned().clone());

        //HashMap转换成Json对象
        let request_body = serde_json::json!(request_body);
        //println!("{:#?},{:#?}", request_heads,request_body);
        //发起消息调用接口请求
        let client = Client::new();
        let sendmsg = client
            .post("https://api.dingtalk.com/v1.0/robot/oToMessages/batchSend")
            .header(request_heads[0].clone(), request_heads[1].clone())
            .json(&request_body)
            .send()
            .await
            .unwrap()
            .text()
            .await;
        info!(
            "send smscode result:{:?},userphone:{}",
            sendmsg, self.userphone
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessApproval<'r> {
    exeuser: &'r str,
    flownumber: &'r str,
    access_token: Option<&'r str>,
    userphone: &'r str,
    userid: Option<&'r str>,
    robotcode: &'r str,
    msgkey: &'r str,
    msgparams: &'r str,
}

impl<'r> ProcessApproval<'r> {
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
    ) -> ProcessApproval<'r> {
        ProcessApproval {
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
    #[allow(unused)]
    pub async fn send_msg(&mut self) {
        //创建请求表头结构
        let mut request_heads: Vec<String> = Vec::new();
        request_heads.push("x-acs-dingtalk-access-token".to_owned());
        request_heads.push(self.access_token.unwrap().to_string());

        //创建请求表体结构
        let mut request_body = HashMap::new();
        request_body.insert("msgParam", self.msgparams);
        request_body.insert("msgKey", self.msgkey);
        request_body.insert("robotCode", self.robotcode);
        request_body.insert("userIds", self.userid.as_ref().unwrap());

        //HashMap转换成Json对象
        let request_body = serde_json::json!(request_body);
        //println!("{:#?},{:#?}", request_heads,request_body);
        //发起消息调用接口请求
        let client = Client::new();
        let sendmsg = client
            .post("https://api.dingtalk.com/v1.0/robot/oToMessages/batchSend")
            .header(request_heads[0].clone(), request_heads[1].clone())
            .json(&request_body)
            .send()
            .await
            .unwrap()
            .text()
            .await;
        info!(
            "send_result{:?},userphone:{}",
            sendmsg,
            self.userid.as_ref().unwrap()
        );
    }
}

#[derive(Debug, Serialize)]
struct Userid<'r> {
    username: &'r str,
    userphone: &'r str,
    dduserid: Option<String>,
}

impl<'r> Userid<'r> {
    pub fn new(username: &'r str, userphone: &'r str) -> Self {
        Userid {
            username,
            userphone,
            dduserid: None,
        }
    }
}

//用来实现消息发送时调用的方法
pub struct SendMSG;

impl SendMSG {
    pub fn new() -> SendMSG {
        SendMSG
    }
    //返回数据库连接配置
    pub fn conn_str(&self) -> String {
        let host = "47.103.31.8";
        let database = "Kxs_Interface";
        let user = "kxs_dev";
        let pwd = "kephi";
        format!(
            r#"Server={};Database={};Uid={};Pwd="{}";TrustServerCertificate=true;"#, //integratedsecurity=sspi 用于进行本地用户验证，不需要user,pwd
            host, database, user, pwd
        )
    }
    //返回数据库连接池
    pub fn buildpools(&self, max_size: u32, min_idle: u32) -> Result<Pool> {
        let pools = mssql::Pool::builder()
            .max_size(max_size)
            .idle_timeout(30 * 60)
            .min_idle(min_idle)
            .max_lifetime(60 * 60 * 2)
            .build(&self.conn_str())
            .unwrap();
        Ok(pools)
    }
    //查询当前待办流程需要发送消息的行数
    pub async fn get_send_num(&self, pools: &Pool) -> i32 {
        //获取数据库连接
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

    //获取userid表中未有userid的用户并回写useid(is working)
    pub async fn get_userlist<'r>(
        &self,
        pools: &Pool,
        gzym_access_token: &'r str,

        zb_access_token: &'r str,
    ) {
        //创建userid列表
        let mut userid_list: Vec<Userid> = Vec::new();
        //获取数据库连接
        let conn = pools.get().await.unwrap();
        //查询用户列表中未更新userid的用户
        let result: Vec<Row> = conn
            .query_collect_row(
               sql_bind!("SELECT username,rtrim(ltrim(userphone))as userphone,rtrim(ltrim(ddUserID))dduserid FROM sendMsg_users WITH(NOLOCK) where isnull(ddUserID,@P1)= @P1","") 
            )
            .await
            .unwrap();

        //将查询未更新userid的列表用户添加到userid_list对象
        for userid in result.iter() {
            userid_list.push(Userid::new(
                userid.try_get_str(0).unwrap().unwrap(),
                userid.try_get_str(1).unwrap().unwrap(),
            ));
        }
        // println!("{:#?}", userid_list);

        //初始化获取userid的对象
        let dduserid = DDUserid::new();

        //遍历userid_list用户，获取userid并更新回数据表
        for user in userid_list.iter_mut() {
            let userid = Some(dduserid.get_userid(gzym_access_token, user.userphone).await);
            match userid {
                Some(v) => {
                    if v.len() > 0 {
                        let _exec = conn
                            .exec(sql_bind!(
                                "UPDATE dbo.sendMsg_users SET dduserid =  @p1,ddtoken = 'gzym_access_token',robotcode='dingrw2omtorwpetxqop' WHERE  userphone = @p2",
                                format!("[\"{}\"]",v),
                                user.userphone
                            ))
                            .await
                            .unwrap();
                    } else {
                        let v = Some(dduserid.get_userid(zb_access_token, user.userphone).await);
                        let _exec = conn
                            .exec(sql_bind!(
                                "UPDATE dbo.sendMsg_users SET dduserid =  @p1,ddtoken = 'zb_access_token',robotcode='dingzblrl7qs6pkygqcn'  WHERE  userphone = @p2",
                                format!("[\"{}\"]",v.unwrap()),
                                user.userphone
                            ))
                            .await
                            .unwrap();
                    }
                }
                _ => {
                    println!("用户{}获取userid失败", user.userphone);
                }
            }
        }
    }

    //执行消息发送
    pub async fn execute_send_msgs<'r>(
        &self,
        pools: &Pool,
        gzym_access_token: &'r str,
        zb_access_token: &'r str,
    ) {
        //创建用户对象列表
        let mut user_list: Vec<ProcessApproval> = Vec::new();
        //获取连接
        let conn = pools.get().await.unwrap();

        info!("{},{}", gzym_access_token, zb_access_token);
        let result: Vec<Row> = conn.query_collect_row("EXEC get_sendmsg").await.unwrap();

        #[allow(unused)]
        let mut access_token = "";
        //遍历查询结果并赋值到user类待发送列表
        for row in result.iter() {
            if let Some("gzym_access_token") = row.try_get_str(2).unwrap() {
                //info!("{}",gzym_access_token);
                access_token = gzym_access_token;
            } else {
                //info!("{}",zb_access_token);
                access_token = zb_access_token;
            };
            user_list.push(ProcessApproval::new(
                row.try_get_str(0).unwrap().unwrap(),
                row.try_get_str(1).unwrap().unwrap(),
                Some(access_token),
                row.try_get_str(3).unwrap().unwrap(),
                row.try_get_str(4).unwrap(),
                row.try_get_str(5).unwrap().unwrap(),
                row.try_get_str(6).unwrap().unwrap(),
                row.try_get_str(7).unwrap().unwrap(),
            ));
        }
        //遍历user列表调用发送方法
        for user in user_list.iter_mut() {
            info!("{:#?}", user);
            // user.send_msg().await;
        }
        //消息发送完成请回写已发送消息项为已发送
        let write_row = conn
            .query_scalar_i32(
                "DECLARE @num INT UPDATE dbo.SendMessage SET rn = '1'  WHERE ISNULL(rn,0) <> 1  SET @num = @@ROWCOUNT SELECT @num",
            )
            .await
            .unwrap()
            .unwrap();
        info!("write back nums:{}.", write_row);
    }
}

//该方法是对消息操作方法的封装（is working）
pub async fn local_thread() {
    let sendmsg = SendMSG::new();
    //获取一个数据连接池对象
    let pools = sendmsg.buildpools(2, 1).unwrap();
    info!("获取连接池成功");
    //判断是否用户列表中有新的用户需要增加到消息发送用户列表
    let conn = pools.get().await.unwrap();
    //先获取当前消息用户列表中是否存在没有DDuserid的用户，有的话查询出来并从钉钉接口中获取usreid
    let addNewUsers = conn
        .query_scalar_i32("DECLARE @row INT EXEC  CheckForNewAddedUsers @row OUTPUT SELECT @row ")
        .await
        .unwrap()
        .unwrap();
    //判断需要获取dduserid的用户大于0
    if addNewUsers > 0 {
        info!("用户列表中dduserid为空的用户数:{}", addNewUsers);
        //初始化广州野马获取access_token的对象
        let gzym_ddtoken = DDToken::new(
            "https://oapi.dingtalk.com/gettoken",
            "dingrw2omtorwpetxqop",
            "Bcrn5u6p5pQg7RvLDuCP71VjIF4ZxuEBEO6kMiwZMKXXZ5AxQl_I_9iJD0u4EQ-N",
        );

        //广州野马获取实时access_token
        let gzym_access_token = gzym_ddtoken.get_token().await;
        info!(
            "广州野马Token:{},robotcode:dingrw2omtorwpetxqop",
            gzym_access_token
        );

        //初始化总部获取access_token的对象
        let zb_ddtoken = DDToken::new(
            "https://oapi.dingtalk.com/gettoken",
            "dingzblrl7qs6pkygqcn",
            "26GGYRR_UD1VpHxDBYVixYvxbPGDBsY5lUB8DcRqpSgO4zZax427woZTmmODX4oU",
        );

        //总部获取实时access_token
        let zb_access_token = zb_ddtoken.get_token().await;
        info!(
            "福建快先森Token:{},robotcode:dingzblrl7qs6pkygqcn",
            zb_access_token
        );
        //循环遍历用户列表中未有dduserid的用户,并回写到消息用户列表中
        sendmsg
            .get_userlist(&pools, &gzym_access_token, &zb_access_token)
            .await;
    }
}
