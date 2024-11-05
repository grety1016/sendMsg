//引入JWT模块
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header as JWTHeader, Validation,
};

use rocket::{
    data::{Data, ToByteUnit},
    fairing::{self, Fairing, Kind},
    http::{
        uri::{self, Origin},
        Header as RocketHeader, Method, Status,
    },
    request::Outcome,
    response::Responder,
    serde::json::Json,
    uri, FromForm, Request, Response,
};

//Hash加密库:
pub use crypto::{digest::Digest, sha2::Sha256};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::field; //用于结构体上方的系列化宏

use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub enum StatusCode {
    Success = 200,
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    UnprocessableEntity = 422,
    TooManyRequests = 429,
    RequestEntityNull = 498,
    RequestEntityNotMatch = 499,
    InternalServerError = 500,
}

//创建CustomResponse结构体,用于返回除了请求路由返回正确数据外的异常信息
#[derive(Serialize, Deserialize, Debug)]
pub struct CstResponse {
    pub Code: i32, // 0：成功，非0：失败
    pub ErrMsg: String,
}
impl CstResponse {
    pub fn new(code: i32, errMsg: String) -> Self {
        CstResponse {
            Code: code,
            ErrMsg: errMsg,
        }
    }
}

//创建Response枚举
#[derive(Responder, Debug)]
pub enum ApiResponse<T: Serialize> {
    #[response(status = 200, content_type = "json")]
    Success(Json<T>),
    #[response(status = 400, content_type = "json")]
    BadRequest(Json<T>),
    #[response(status = 401, content_type = "json")]
    Unauthorized(Json<T>),
    #[response(status = 403, content_type = "json")]
    Forbidden(Json<T>),
    #[response(status = 404, content_type = "json")]
    NotFound(Json<T>),
    #[response(status = 405, content_type = "json")]
    MethodNotAllowed(Json<T>),
    #[response(status = 422, content_type = "json")]
    TooManyRequests(Json<T>),
    #[response(status = 429, content_type = "json")]
    UnprocessableEntity(Json<T>),
    #[response(status = 500, content_type = "json")]
    InternalServerError(Json<T>),
}

/// Token验证Fairinge
pub struct TokenFairing;
//Token Fairing实现
#[rocket::async_trait]
impl Fairing for TokenFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Token validation",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        // println!("{}", req.uri().path());
        // println!("{:#?}", req);
        //为了匹配路由url将所有的地址全部转换为小写
        let uri = req.uri().to_string().to_lowercase();
        //将url转换成小写后回写到请求中
        let url = Origin::try_from(uri.clone()).unwrap();
        req.set_uri(url);

        //记录下用户原始请求的URL
        let originURL = format!("http://{}{}", req.host().unwrap(), uri);
        req.add_header(RocketHeader::new("originURL", originURL));
        //判断一下是否是短信验证码接口，如果是，则无需验证token,直接请求路由
        if uri.starts_with("/user/getsmscode") || uri.starts_with("/files") {
            return;
        }

        //从表头中读取token字段，并验证token是否有效
        let token = req.headers().get_one("Authorization");
        let mut verifyResult: bool = false;
        if let Some(value) = token {
            verifyResult = Claims::verify_token(value.to_string()).await;
        }

        //token验证成功
        if verifyResult {
            //判断是否访问主页，如果是，则重定向到登录页面
            if *req.uri() == "/login"
                || (*req.uri() == "/user/login" && req.method() == Method::Get)
            {
                let url = Origin::try_from("/").unwrap();
                req.set_uri(url);
            }
            return;
        } else {
            //token验证失败
            // req.add_header(RocketHeader::new("Custom-Header", "Unauthorized"));
            let url = Origin::parse("/user/login").unwrap();
            // req.set_method(Method::Get);
            req.set_uri(url);
            return;
        }
        // }
    }
    /*************************************************************************************
    如上代码用于验证token，并且是POST方法才生效*/
    // async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
    //     println!("{:#?},{:#?}", res.status(), res.body());
    //     println!("请求信息：\n{:#?}", req);
    //     println!("响应信息：\n{:#?}", res);

    // }
}

//创建JWT结构体
#[derive(Serialize, Deserialize, Debug, Clone, FromForm)] //FromForm用于从数据库返回不区分大小写
pub struct LoginUser {
    #[field(name=uncase("userphone"))]
    pub userPhone: String,
    #[field(name=uncase("smscode"))]
    pub smsCode: String,
    pub token: String,
}
//创建LoginResponse结构体
#[derive(Serialize, Deserialize, Debug, FromForm)]
pub struct LoginResponse {
    #[field(name=uncase("userphone"))]
    pub userPhone: String,
    #[field(name=uncase("smscode"))]
    pub smsCode: i32,
    pub token: String,
    pub code: i32, // 0：成功，非0：失败
    #[field(name=uncase("errmsg"))]
    pub errMsg: String,
}
impl LoginResponse {
    pub fn new(token: String, data: LoginUser, code: i32, errMsg: String) -> Self {
        LoginResponse {
            code,
            token,
            userPhone: data.userPhone,
            smsCode: 0,
            errMsg,
        }
    }
}

//创建FlowForm返回结构体
#[derive(Serialize, Deserialize, Debug)]
pub struct FlowItemList {
    eventName: String, //事项名称
    rn: i32,
    fStatus: String,        //（运行中、挂起、终止、暂停）
    fNumber: String,        //实例编码
    fFormID: String,        //流程单据ID
    fFormType: String,      //流程单据类型
    fDisplayName: String,   //实例名称
    todoStatus: i32,        //处理状态（未处理、已处理）0：未处理 1：已处理  2：我发起
    fName: String,          //流程发起者
    senderPhone: String,    //发起者电话
    fReceiverNames: String, //流程接收者
    fPhone: String,
    ///接收者手机号
    fProcinstID: String, //实例内码
    fCreateTime: String, //流程创建日期
}

//创建FlowForm明细信息结构体(费用报销单-差旅报销单)
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetailFybxAndClbx {
    pub available: i32,      //是否有效流程
    fBillNo: String,         // 流程编码
    fOrgID: String,          // 申请组织
    fRequestDeptID: String,  // 申请部门
    fProposerID: String,     // 申请人
    fExpenseOrgID: String,   // 费用组织
    fExpenseDeptID: String,  // 费用部门
    fCurrency: String,       // 币别
    fReqReimbAmountSum: f64, // 申请报销金额汇总
    fExpAmountSum: f64,      // 核定报销金额
    fCausa: String,          // 事由
    years: String,           // 年份
    status: String,          //状态
}

//创建流程表单明细报销明细行结构体（费用报销单）
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetailRowFybx {
    pub attachments: Option<Vec<Attachments>>, //附件列表
    pub fSnnaAttachments: String,              //附件字符串
    pub fName: String,                         //费用项目
    pub fExpenseAmount: f64,                   //申请金额
    pub fExpSubmitAmount: f64,                 //核定金额
    pub years: String,                         //年份
}
//创建流程表单明细报销明细行结构体（差旅报销单）
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetailRowClbx {
    pub attachments: Option<Vec<Attachments>>, //附件列表
    pub fSnnaAttachments: String,              //附件字符串
    pub fClType: String,                       //差旅费类型
    pub fTravelAmount: f64,                    //差旅费金额
    pub fName: String,                         //费用项目
    pub fExpenseAmount: f64,                   //申请金额
    pub fExpSubmitAmount: f64,                 //核定金额                         //费用类型
    pub years: String,                         //年份
}

//创建流程表单明细行结构体-附件结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Attachments {
    pub ServerFileName: String,   //文件地址
    pub FileName: String,         //文件名称
    pub FileLength: f64,          //
    pub FileBytesLength: f64,     //
    pub FileType: Option<String>, //文件类型
    pub FileSize: Option<String>, //文件大小
}

//创建FlowForm明细信息结构体(费用申请单)
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetailFysqAndCcsq {
    pub available: i32,      //是否有效流程
    fBillNo: String,         // 流程编码
    fOrgID: String,          // 申请组织
    fRequestDeptID: String,  // 申请部门
    fProposerID: String,     // 申请人
    fExpenseOrgID: String,   // 费用组织
    fExpenseDeptID: String,  // 费用部门
    fIsBorrow: String,       // 是否借款
    fCurrency: String,       // 币别
    fReqReimbAmountSum: f64, // 申请金额汇总
    fExpAmountSum: f64,      // 核定金额汇总
    fCausa: String,          // 事由
    years: String,           // 年份
    status: String,          //状态
}

//创建流程表单明细报销明细行结构体（费用申请单）
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetailRowFysq {
    pub fName: String,            //费用项目
    pub fExpenseAmount: f64,      //申请金额
    pub fExpSubmitAmount: String, //核定金额
    pub fStartDate: String,       //开始日期
    pub years: String,            //年份
}
//创建流程表单明细报销明细行结构体（出差申请单）
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetailRowCcsq {
    pub fName: String,            //费用项目
    pub fExpenseAmount: f64,      //申请金额
    pub fExpSubmitAmount: String, //核定金额
    pub fStartDate: String,       //开始日期
    pub fEndDate: String,
    pub fTtravelStartSite: String, //出发地
    pub fTravelEndSite: String,    //目的地
    pub fAirTicketCost: f64,       //机票
    pub fOtherRemoteCost: f64,     //其他长途费用
    pub fFlocalCost: f64,          //市内交通费
    pub fAccomFee: f64,            //住宿费
    pub fOtherExpense: f64,        //其他费用
    pub fTravelSubsidy: f64,       //差旅补贴
    pub years: String,             //年份
}

//创建JWT结构体
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
}

//接收文本消息结构中的文字
#[derive(Debug, Deserialize, Serialize)]
pub struct Text {
    pub content: String,
}
//接收消息文本结构
#[derive(Debug, Deserialize, Serialize)]
pub struct RecvMessage {
    pub senderStaffId: String,
    pub text: Option<Text>,
    pub content: Option<Content>,
    pub msgtype: String,
}
//接收语音消息结构中的文字
#[derive(Debug, Deserialize, Serialize)]
pub struct Content {
    pub recognition: String,
}

impl Claims {
    pub fn new(sub: String) -> Self {
        let nowTimeStamp = jsonwebtoken::get_current_timestamp();
        let exp = nowTimeStamp + 31 * 24 * 60 * 60; //设置token过期时间为一个月
        Claims { sub, exp }
    }

    pub async fn get_token(usrPhone: String) -> String {
        let mut secretKey =
            env::var("TokenSecretKey").unwrap_or_else(|_| String::from("kephi520."));

        let mut hasherSecretKey = Sha256::new();
        hasherSecretKey.input_str(secretKey.as_ref());
        secretKey = hasherSecretKey.result_str();

        let claims = Claims::new(usrPhone.to_owned());

        let token = encode(
            &JWTHeader::default(),
            &claims,
            &EncodingKey::from_secret(secretKey.as_ref()),
        )
        .unwrap();
        // println!("token:{:#?}", token);
        token
    }
    pub async fn verify_token(token: String) -> bool {
        let mut secretKey = env::var("TokenSecretKey").unwrap_or(String::from("kephi520."));

        let mut hasherSecretKey = Sha256::new();
        hasherSecretKey.input_str(secretKey.as_ref());
        secretKey = hasherSecretKey.result_str();

        let mut validate = Validation::new(Algorithm::HS256);
        validate.leeway = 0; //设置偏差为0

        let deToken = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secretKey.as_ref()),
            &validate,
        );
        match deToken.is_ok() {
            true => true,
            false => false,
        }
    }
    pub async fn get_phone(token: String) -> String {
        let mut secretKey =
            env::var("TokenSecretKey").unwrap_or_else(|_| String::from("kephi520."));

        let mut hasherSecretKey = Sha256::new();
        hasherSecretKey.input_str(secretKey.as_ref());
        secretKey = hasherSecretKey.result_str();

        let mut validate = Validation::new(Algorithm::HS256);
        validate.leeway = 0; //设置偏差为0

        let deToken = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secretKey.as_ref()),
            &validate,
        );
        //println!("{:#?}", deToken);
        match deToken {
            Ok(token) => token.claims.sub,
            Err(_) => "".to_string(),
        }
    }
}
