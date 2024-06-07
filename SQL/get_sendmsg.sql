ALTER PROC get_sendmsg
AS
BEGIN
  SELECT EXEUSER,FLOWNUMBER,access_token,userphone,userid,robotcode,msgkey,'{"text": "���� ' + CAST(msgparams AS varchar(10)) +' ��������̴��죬�뼰ʱǰ������ͻ��˻򶤶�����̨-ҵ����������","title": "������̴���"}' FROM (
		SELECT MAX(exeuser) as exeuser,
			   max(flownumber) as flownumber,
			   max(RTRIM(T2.access_token)) as access_token,
			   max(T2.userphone) as userphone,
			   max(T2.userid) as userid,
			   max(RTRIM(T2.robotcode)) as robotcode,
			   max(RTRIM(msgkey)) as msgkey,
			   count(msgparams) as msgparams
		FROM SendMessage T1 WITH(NOLOCK)
		inner join (SELECT distinct userphone,username,userid,access_token,robotcode FROM UserID WITH (NOLOCK) ) T2 ON T1.userphone = T2.userphone
		where ISNULL(t1.rn,'')<>1
		GROUP BY T1.userphone) T 
END;
 

 
 