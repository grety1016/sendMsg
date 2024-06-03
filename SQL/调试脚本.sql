--创建消息列表数据表
-- CREATE TABLE [dbo].[SendMessage](
--	[exeuser] [nvarchar](50) NOT NULL,
--	[flownumber] [nvarchar](100) NOT NULL,
--	[access_token] [char](100) NOT NULL,
--	[userphone] [char](20) NOT NULL,
--	[userid] [char](50) NOT NULL,
--	[robotcode] [char](30) NOT NULL,
--	[msgkey] [char](50) NOT NULL,
--	[msgparams] [nvarchar](500) NOT NULL,
--	[rn] [int] NULL,
--	[fcreatetime] [datetime] not NULL,
--	primary key (flownumber,fcreatetime)
--)  
--创建用户userid表
-- CREATE TABLE [dbo].[UserID](
--	[userphone] [nvarchar](20) NOT NULL,
--	[userid] [nvarchar](50)  NULL,	
--	primary key (userphone)
--)  
  
--执行获取消息列表过程
BEGIN TRAN --rollback commit
DECLARE @num INT
EXEC  @num = get_flow_list
SELECT @num

--查询消息发送列表 
SELECT * FROM SendMessage

--删除消息列表数据
DELETE dbo.SendMessage


--执行获取useid列表过程
BEGIN TRAN --rollback commit
DECLARE @num2 INT
EXEC  @num2 = insert_userid_table
SELECT @num2

--查询消息发送列表 
SELECT username,trim(userphone)userphone,userid FROM UserID 
WHERE isnull(userid,'')=''
SELECT username,trim(userphone)userphone, userid  FROM UserID 
WHERE username IN('崔雅','祝延男','邓祥华','林晶')

SELECT username,rtrim(ltrim(userphone)),rtrim(ltrim(userid)) FROM UserID WITH(NOLOCK) where isnull(userid,'')=''

 --更新用户表的userid
 UPDATE dbo.UserID SET userid = '' WHERE  username IN('陈丹丹','陈梅业')

 UPDATE dbo.UserID SET userphone = '19075398396' WHERE  userphone = '19539245142'


 --删除未在钉钉中的用户
 DELETE UserID WHERE  userphone = '19126493775'

 --查询用户名手机
 SELECT *FROM UserID WITH(NOLOCK)  
 WHERE  username IN('陈丹丹','陈梅业')

  SELECT FNAME,FPHONE,'' FROM t_sec_user WITH(NOLOCK)
  WHERE FNAME IN('陈丹丹','陈梅业')

  UPDATE t_sec_user SET fphone = '17805938219' WHERE fname='陈丹丹'

  UPDATE t_sec_user SET fphone = '19075398396' WHERE fname='陈梅业'


--删除消息列表数据
DELETE dbo.UserID WHERE  username IN('陈丹丹','陈梅业')

DROP TABLE dbo.UserID
 
 --更新某条流程的最近处理时间
 update T_WF_ASSIGN
 set fcreatetime =  getdate()
 where fyzjmsgid IN('b5d1d855-3334-4350-baed-652cb8575b90','2083bf64-79b5-4d14-9a5e-718a1f61c429')

 --查询当前需要调整时间的流程
 SELECT b.FNUMBER,a.FCREATETIME,* FROM T_WF_ASSIGN a
 LEFT JOIN T_WF_PROCINST b ON a.FPROCINSTID = b.FPROCINSTID
 SELECT FNUMBER,FCREATETIME,* FROM V_WF_ASSIGN

 --查询用户表


 
 
 
 
	 


	
SELECT  max(exeuser),flownumber,access_token,userphone,userid,robotcode,msgkey,msgparams,rn FROM SendMessage  
group by userphone