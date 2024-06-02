 ALTER PROC  get_sendmsg(@username NVARCHAR(50))
 AS 
 BEGIN 
	 SELECT username AS exeuser,'001_20240525161051' AS flownumber,'' AS access_token,userphone,userid,'dingrw2omtorwpetxqop' AS robotcode,'sampleMarkdown' AS msgkey,
 '{"text": "您有一条消息待办，请前往金蝶客户端或钉钉移动端处理！","title": "金蝶流程提醒"}' AS msgparams FROM dbo.UserID u WITH(NOLOCK)
  WHERE (u.username = @username OR ISNULL(@username,'')='')
 END 

 