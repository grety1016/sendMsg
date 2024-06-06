ALTER PROCEDURE dbo.get_flow_list
AS
BEGIN
    DECLARE @effect INT;
    BEGIN TRAN; --rollback
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
    SELECT u.FNAME exeuser,
           a.FNUMBER flownumber,
           '' AS access_token,
           u.FPHONE userphone,
           '' AS userid,
           'dingrw2omtorwpetxqop' AS robotcode,
           'sampleMarkdown' AS msgkey,
           '{"text": "您有一条消息待办，请前往金蝶客户端或钉钉移动端处理！","title": "金蝶流程提醒"}' AS msgparams,
           a.FCREATETIME,
           0
    FROM V_WF_ASSIGN a WITH (NOLOCK)
        INNER JOIN T_SEC_USER u WITH (NOLOCK)
            ON a.FRECEIVERID = u.FUSERID
    WHERE (a.FNUMBER NOT IN
           (
               SELECT flownumber FROM dbo.SendMessage sg WITH (NOLOCK)
           )
          )
    UNION ALL
    SELECT u.FNAME exeuser,
           a.FNUMBER flownumber,
           '' AS access_token,
           u.FPHONE userphone,
           '' AS userid,
           'dingrw2omtorwpetxqop' AS robotcode,
           'sampleMarkdown' AS msgkey,
           '{\"text\": \"您有一条消息待办，请前往金蝶客户端或钉钉移动端处理！\",\"title\": \"金蝶流程提醒\"}' AS msgparams,
           a.FCREATETIME,
           0
    FROM V_WF_ASSIGN a WITH (NOLOCK)
        INNER JOIN T_SEC_USER u WITH (NOLOCK)
            ON a.FRECEIVERID = u.FUSERID
        INNER JOIN
        (SELECT DISTINCT flownumber FROM SendMessage WITH (NOLOCK)) sg
            ON a.FNUMBER = sg.flownumber --当流程实例已经在发送列表中出现多次，仅取出其中一条来匹配，按时间戳来判断是否要加入到发送列表
    WHERE a.FCREATETIME NOT IN
          (
              SELECT fcreatetime FROM dbo.SendMessage sg2 WITH (NOLOCK)
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

