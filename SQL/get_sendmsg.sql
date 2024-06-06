ALTER PROC get_sendmsg
(@username NVARCHAR(50))
AS
BEGIN
    SELECT username AS exeuser,
           '' AS flownumber,
           RTRIM(access_token) AS access_token,
           userphone,
           userid,
           CASE
               WHEN access_token = 'gzym_access_token' THEN
                   'dingrw2omtorwpetxqop'
               WHEN access_token = 'zb_access_token' THEN
                   'dingzblrl7qs6pkygqcn'
           END AS robotcode,
           'sampleLink' AS msgkey,
           '{     
				"msgtype": "link",     
				"messageUrl": "http://210ie6ur7254.vicp.fun/?phone=13933611151",        
				 "picUrl":"@lADPDfJ6fUduS0DM8Mzw",        
				 "title": "金蝶待办消息接口测试",        
				 "text": "您好，请点击链接加入快先森金蝶待办消息接口！"     
			}'        AS msgparams
    FROM dbo.UserID u WITH (NOLOCK)
    WHERE (
              u.username = @username
              OR ISNULL(@username, '') = ''
          );
END;

