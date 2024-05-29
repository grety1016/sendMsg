ALTER PROCEDURE dbo.get_flow_list 
AS
BEGIN 
	DECLARE @effect INT
	BEGIN TRAN --rollback
	INSERT INTO dbo.SendMessage
	(
	    exeuser,
	    flownumber,
		access_token,	    
	    userphone,
	    userid,
	    robotcode,
	    msgkey,
	    msgparams,
		fcreatetime,
		rn
	)
	SELECT  
		u.FNAME exeuser,
		a.FNUMBER flownumber,
		'' AS access_token,
		u.FPHONE userphone,
		''AS userid,
		'dingrw2omtorwpetxqop' AS robotcode,
		'sampleMarkdown'AS msgkey,
		'{\"text\": \"����һ����Ϣ���죬��ǰ������ͻ��˻򶤶��ƶ��˴���\",\"title\": \"�����������\"}' AS msgparams,
		a.fcreatetime,
		0
	FROM V_WF_ASSIGN a WITH(NOLOCK)
	INNER JOIN T_SEC_USER u WITH(NOLOCK) ON a.FReceiverId=u.FUSERID
	WHERE (a.FNUMBER NOT IN (SELECT flownumber FROM	dbo.SendMessage sg with(nolock))) 

	UNION ALL

	SELECT  
		u.FNAME exeuser,
		a.FNUMBER flownumber,
		'' AS access_token,
		u.FPHONE userphone,
		''AS userid,
		'dingrw2omtorwpetxqop' AS robotcode,
		'sampleMarkdown'AS msgkey,
		'{\"text\": \"����һ����Ϣ���죬��ǰ������ͻ��˻򶤶��ƶ��˴���\",\"title\": \"�����������\"}' AS msgparams,
		a.fcreatetime,
		0
	FROM V_WF_ASSIGN a WITH(NOLOCK)
	INNER JOIN T_SEC_USER u WITH(NOLOCK) ON a.FReceiverId=u.FUSERID
	INNER JOIN (SELECT DISTINCT flownumber FROM SendMessage) sg on a.fnumber = sg.flownumber --������ʵ���Ѿ��ڷ����б��г��ֶ�Σ���ȡ������һ����ƥ�䣬��ʱ������ж��Ƿ�Ҫ���뵽�����б�
	WHERE a.fcreatetime  NOT IN (SELECT fcreatetime FROM	dbo.SendMessage sg with(nolock))
 



	SET @effect = @@ROWCOUNT

	if @@ERROR = 0 
		BEGIN
			COMMIT 
			RETURN @effect
		END

	ELSE 
		BEGIN
			ROLLBACK    
			RETURN -1
		END
	END;

 