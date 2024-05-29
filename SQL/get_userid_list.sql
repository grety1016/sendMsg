ALTER PROCEDURE dbo.get_userid_list 
AS
BEGIN 
	DECLARE @effect INT

	--查询消息列表中未发送的用户手机是否已经存在useid表中，如果没有，加入userid表
	BEGIN TRAN

	INSERT INTO dbo.UserID
	(
	    userphone
	)	
	SELECT DISTINCT sg.userphone  FROM SendMessage sg
	WHERE ISNULL(sg.rn,0) <> 1 AND  sg.userphone NOT IN  (SELECT DISTINCT userphone FROM userid) 
	
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

 