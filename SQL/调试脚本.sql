--������Ϣ�б����ݱ�
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
--�����û�userid��
-- CREATE TABLE [dbo].[UserID](
--	[userphone] [nvarchar](20) NOT NULL,
--	[userid] [nvarchar](50)  NULL,	
--	primary key (userphone)
--)  
  
--ִ�л�ȡ��Ϣ�б����
BEGIN TRAN --rollback commit
DECLARE @num INT
EXEC  @num = get_flow_list
SELECT @num

--��ѯ��Ϣ�����б� 
SELECT * FROM SendMessage

--ɾ����Ϣ�б�����
DELETE dbo.SendMessage


--ִ�л�ȡuseid�б����
BEGIN TRAN --rollback commit
DECLARE @num2 INT
EXEC  @num2 = insert_userid_table
SELECT @num2

--��ѯ��Ϣ�����б� 
SELECT username,trim(userphone)userphone,userid FROM UserID where isnull(userid,'')=''
SELECT username,trim(userphone)userphone, userid  FROM UserID 
WHERE username IN('����','ף����','���黪','�־�')


 --�����û����userid
 UPDATE dbo.UserID SET userid = '' WHERE  username IN('ף����')

 UPDATE dbo.UserID SET userphone = '18074624411' WHERE  userphone = '19976600793'


 --ɾ��δ�ڶ����е��û�
 DELETE UserID WHERE  userphone = '19126493775'

 --��ѯ�û����ֻ�
 SELECT *FROM UserID WITH(NOLOCK) 

--ɾ����Ϣ�б�����
DELETE dbo.UserID
 
 --����ĳ�����̵��������ʱ��
 update T_WF_ASSIGN
 set fcreatetime =  getdate()
 where fyzjmsgid IN('b5d1d855-3334-4350-baed-652cb8575b90','2083bf64-79b5-4d14-9a5e-718a1f61c429')

 --��ѯ��ǰ��Ҫ����ʱ�������
 SELECT b.FNUMBER,a.FCREATETIME,* FROM T_WF_ASSIGN a
 LEFT JOIN T_WF_PROCINST b ON a.FPROCINSTID = b.FPROCINSTID
 SELECT FNUMBER,FCREATETIME,* FROM V_WF_ASSIGN

 --��ѯ�û���



 
 
 
 
	 


	
SELECT  max(exeuser),flownumber,access_token,userphone,userid,robotcode,msgkey,msgparams,rn FROM SendMessage  
group by userphone