 ALTER PROC  get_sendmsg(@username NVARCHAR(50))
 AS 
 BEGIN 
	 SELECT username AS exeuser,'001_20240525161051' AS flownumber,'' AS access_token,userphone,userid,'dingrw2omtorwpetxqop' AS robotcode,'sampleMarkdown' AS msgkey,
 '{"text": "����һ����Ϣ���죬��ǰ������ͻ��˻򶤶��ƶ��˴���","title": "�����������"}' AS msgparams FROM dbo.UserID u WITH(NOLOCK)
  WHERE (u.username = @username OR ISNULL(@username,'')='')
 END 

 