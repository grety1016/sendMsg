use tokio::time;
//reqwestHTTP请求
use httprequest::Client;
use std::fmt::format;
//hashmap
use std::{collections::HashMap, result};
//系列化
use serde::{Deserialize, Serialize};//用于结构体上方的系列化宏

//日志追踪
pub use tracing::{info,event,warn,trace,Level};


//引入数据库
pub use mssql::Pool;
pub use mssql::*;





//钉钉获取token请求主体
#[derive(Debug, Serialize, Deserialize)]
struct DDToken<'r> {
    url: &'r str,
    appkey: &'r str,
    appsecret: &'r str,
}
//获取token后的结果类型
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
//钉钉DDUserid实现
impl DDUserid {
    pub fn new() -> DDUserid {
        DDUserid
    }

    //调用通过手机获取userid
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
        let mut x = String::new();

        //匹配返回结果，调用成功把值给userid
        match useridresult {
            Ok(v) => {
                x = v.clone();
                //匹配系列化结果，如果结果是可以转换的，转换成json
                match serde_json::from_str(&x) {
                    Ok(v) => userid = v,
                    Err(e) => event!(Level::ERROR,"turn_userid_error: {:?}", e),
                }
            }
            Err(e) => event!(Level::ERROR,"get_userid_error:{:?}", e),
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
        request_heads.push(self.access_token.unwrap().to_string());

        //创建请求表体结构
        let mut request_body = HashMap::new();
        request_body.insert("msgParam", self.msgparams);
        request_body.insert("msgKey", self.msgkey);
        request_body.insert("robotCode", self.robotcode);
        request_body.insert("userIds", self.userid.as_ref().unwrap());

        //HashMap转换成Json对象
        let request_body = serde_json::json!(request_body);
        //println!("{:#?}", request_body);
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
        
    }
}

#[derive(Debug, Serialize)]
struct Userid<'r> {
    username: &'r str,
    userphone: &'r str,
    userid: Option<String>,
}

impl<'r> Userid<'r> {
    pub fn new(username: &'r str, userphone: &'r str) -> Self {
        Userid {
            username,
            userphone,
            userid: None,
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

    //获取userid表中未有userid的用户并回写useid
    pub async fn get_userid_list<'r>(&self, pools: &Pool, access_token: &'r str) {
        //创建userid列表
        let mut userid_list: Vec<Userid> = Vec::new();
        //获取数据库连接
        let conn = pools.get().await.unwrap();
        //执行未添加到用户表插入user列表 
        let insert_users = conn
            .query_scalar_i32(
                "DECLARE @num2 INT
                    EXEC  @num2 = insert_userid_table
                    SELECT @num2",
            )
            .await.unwrap().unwrap();
        info!("innser_users:{:?}", insert_users);
        //查询用户列表中未更新userid的用户
        let result: Vec<Row> = conn
            .query_collect_row(
                "SELECT username,rtrim(ltrim(userphone)),rtrim(ltrim(userid)) FROM UserID WITH(NOLOCK) where isnull(userid,'')=''"
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
    

        //初始化获取userid的对象
        let dduserid = DDUserid::new(); 

        //遍历userid_list用户，获取userid并更新回数据表
        for user in userid_list.iter_mut() {

            let userid = Some(dduserid.get_userid(access_token, user.userphone).await);             
                let _exec = conn
                    .exec(sql_bind!(
                        "UPDATE dbo.UserID SET userid = @p1 WHERE  userphone = @p2",
                        //userid.as_ref().unwrap(),
                        userid.as_ref().unwrap().clone(),
                        user.userphone)
                    )
                    .await
                    .unwrap();
             
        }

        //更新userid表内userid为数组格式
    }

    //执行消息发送
    pub async fn execute_send_msgs<'r>(&self, pools: &Pool, access_token: &'r str) {
        //创建用户对象列表
        let mut user_list: Vec<User> = Vec::new();
        //获取连接
        let conn = pools.get().await.unwrap();

        let result: Vec<Row> = conn
            .query_collect_row("EXEC get_sendmsg @username='苏宁绿'")
            .await
            .unwrap();
        for row in result.iter() {
            user_list.push(User::new(
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

        for user in user_list.iter_mut() {
            //println!("{:#?}",user);
            user.send_msg().await;
        }
    }
}


//该方法是对消息操作方法的封装
pub async fn local_thread() {
    let sendmsg = SendMSG::new();
    //获取一个数据连接池对象
    let pools = sendmsg.buildpools().unwrap();

    //获取数据库中待办满足发送消息的流程数量
    let sendmsgnum = sendmsg.get_send_num(&pools).await;
    info!("获取到需发送的列表用户数：{}", sendmsgnum);

    //初始化广州野马获取access_token的对象
    let gzym_ddtoken = DDToken::new(
        "https://oapi.dingtalk.com/gettoken",
        "dingrw2omtorwpetxqop",
        "Bcrn5u6p5pQg7RvLDuCP71VjIF4ZxuEBEO6kMiwZMKXXZ5AxQl_I_9iJD0u4EQ-N",
    );

    //广州野马获取实时access_token
    let gzym_access_token = gzym_ddtoken.get_token().await;
    info!("gzym_access_token:{}", gzym_access_token);

    //初始化总部获取access_token的对象
    let zb_ddtoken = DDToken::new(
        "https://oapi.dingtalk.com/gettoken",
        "dingzblrl7qs6pkygqcn",
        "26GGYRR_UD1VpHxDBYVixYvxbPGDBsY5lUB8DcRqpSgO4zZax427woZTmmODX4oU",
    );

    //总部获取实时access_token
    let zb_access_token = zb_ddtoken.get_token().await;
    info!("zb_access_token:{}", zb_access_token);

    //广州野马获取userid
    sendmsg.get_userid_list(&pools, &gzym_access_token).await;

    //总部获取userid
    sendmsg.get_userid_list(&pools, &zb_access_token).await;

}
