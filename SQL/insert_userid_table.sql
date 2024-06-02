ALTER PROCEDURE dbo.insert_userid_table
AS
BEGIN 
	DECLARE @effect INT

	--��ѯ��Ϣ�б���δ���͵��û��ֻ��Ƿ��Ѿ�����useid���У����û�У�����userid��
	BEGIN TRAN

	INSERT INTO dbo.UserID
	(
		username,
	    userphone,
		userid
	)	
	
 SELECT FNAME,FPHONE,'' FROM t_sec_user WITH(NOLOCK) WHERE FForbidStatus <>'B'   AND  
	FNAME NOT IN ('guest','attendance','administrator','demo','demo1','demo2')
	AND FPHONE NOT IN (SELECT DISTINCT userphone FROM UserID WITH(NOLOCK))
	
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
	
END

 