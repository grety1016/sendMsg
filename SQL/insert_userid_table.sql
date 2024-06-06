ALTER PROCEDURE dbo.insert_userid_table
AS
BEGIN
    DECLARE @effect INT;

    --��ѯ��Ϣ�б���δ���͵��û��ֻ��Ƿ��Ѿ�����useid���У����û�У�����userid��
    BEGIN TRAN;

    INSERT INTO dbo.UserID
    (
        username,
        userphone,
        userid
    )
    SELECT FNAME,
           FPHONE,
           ''
    FROM T_SEC_USER WITH (NOLOCK)
    WHERE FFORBIDSTATUS <> 'B'
          AND FNAME NOT IN ( 'guest', 'attendance', 'administrator', 'demo', 'demo1', 'demo2', '��𽡿�', '���黪', '�����',
                             '���޺�', '������', 'κ�ӳ�', '������', '��ï��', '����'
                           )
          AND FPHONE NOT IN
              (
                  SELECT DISTINCT userphone FROM UserID WITH (NOLOCK)
              );

    SET @effect = @@ROWCOUNT;

    IF @@ERROR = 0
    BEGIN
        COMMIT;
        RETURN @effect;
    END;

    ELSE
    BEGIN
        ROLLBACK;
        RETURN -1;
    END;

END;

