use crate::{Connection, Error, Result, ResultSet};
use std::{borrow::Cow, mem::transmute, time};
use tokio::time::timeout;

/// SQL语句
#[derive(Debug)]
pub struct Sql<'a>(tiberius::Query<'a>);

impl<'a> Sql<'a> {
    /// 创建参数化SQL
    pub fn new(sql: impl Into<Cow<'a, str>>) -> Self { Sql(tiberius::Query::new(sql)) }
    /// 绑定参数 `@PN`
    pub fn bind(&mut self, param: impl tiberius::IntoSql<'a> + 'a) { self.0.bind(param); }
    /// 预览语句
    pub fn preview(&self) -> String {
        lazy_static::lazy_static! {
            static ref RE_SQL_LOG_FMT: regex::Regex = regex::Regex::new(r"\s+").unwrap();
        }
        let sql = format!("{:?}", self.0);
        //格式化输出的SQL,使输出更紧凑
        RE_SQL_LOG_FMT.replace_all(&sql, " ").into_owned()
    }
    /// 执行命令
    ///
    /// # Parameters
    ///
    /// - **conn** 连接
    /// - **duration** 超时时间
    ///
    /// # Returns
    ///
    /// 返回影响的行数
    pub(crate) async fn exec(self, conn: &Connection, duration: Option<time::Duration>) -> Result<u64> {
        if conn.is_pending() {
            return Err(Error::PendingError);
        }
        let sql_preview = self.preview();
        info!(
            "#{}> [{}] @{} - executing, sql: {}",
            conn.spid(),
            conn.log_category(),
            conn.log_db_name(),
            sql_preview
        );
        conn.set_pending(true);
        let now = time::Instant::now();
        let mut raw = conn.raw_ref().lock().await;
        let rv = if let Some(duration) = duration {
            timeout(duration, self.0.execute(&mut raw))
                .await
                .map_err(|_| Error::ExecTimeout)
                .and_then(|rv| rv.map_err(Error::ExecError))
        } else {
            self.0.execute(&mut raw).await.map_err(Error::ExecError)
        };
        match rv {
            Ok(rs) => {
                conn.set_pending(false);
                //FIXME
                //**BUG**
                //取数组最后一个值
                let effected = *rs.rows_affected().last().unwrap_or(&0); //rv.total();
                info!(
                    "#{}> [{}] @{} - elapsed: {}ms, effected: {}, sql: {}",
                    conn.spid(),
                    conn.log_category(),
                    conn.log_db_name(),
                    now.elapsed().as_millis(),
                    effected,
                    sql_preview
                );
                Ok(effected)
            },
            Err(e) => {
                if matches!(e, Error::ExecTimeout) {
                    warn!(
                        "#{}> [{}] @{} - elapsed: {}ms, timeout, sql: {}",
                        conn.spid(),
                        conn.log_category(),
                        conn.log_db_name(),
                        now.elapsed().as_millis(),
                        sql_preview
                    );
                    //FIXME
                    //自动重连
                    conn.reconnect().await?;
                } else {
                    conn.set_pending(false);
                    warn!(
                        "#{}> [{}] @{} - elapsed: {}ms, error: {}, sql: {}",
                        conn.spid(),
                        conn.log_category(),
                        conn.log_db_name(),
                        now.elapsed().as_millis(),
                        e,
                        sql_preview
                    );
                }
                Err(e)
            }
        }
    }
    /// 查询语句
    ///
    /// # Parameters
    ///
    /// - **conn** 连接
    /// - **duration** 超时时间
    ///
    /// # Returns
    ///
    /// 返回结果集对象
    pub(crate) async fn query<'con>(
        self,
        conn: &'con Connection,
        duration: Option<time::Duration>
    ) -> Result<ResultSet<'con>> {
        if conn.is_pending() {
            return Err(Error::PendingError);
        }
        let sql_preview = self.preview();
        info!(
            "#{}> [{}] @{} - executing, sql: {}",
            conn.spid(),
            conn.log_category(),
            conn.log_db_name(),
            sql_preview
        );
        conn.set_pending(true);
        let now = time::Instant::now();
        let mut raw = conn.raw_ref().lock().await;
        let rv = if let Some(duration) = duration {
            timeout(duration, self.0.query(&mut raw))
                .await
                .map_err(|_| Error::QueryTimeout)
                .and_then(|rv| rv.map_err(Error::QueryError))
        } else {
            self.0.query(&mut raw).await.map_err(Error::QueryError)
        };
        let rv = match rv {
            Ok(rs) => ResultSet::new(rs, conn.alive_rs_ref()).await,
            Err(e) => Err(e)
        };
        match rv {
            Ok(rs) => {
                conn.set_pending(false);
                info!(
                    "#{}> [{}] @{} - elapsed: {}ms, sql: {}",
                    conn.spid(),
                    conn.log_category(),
                    conn.log_db_name(),
                    now.elapsed().as_millis(),
                    sql_preview
                );
                //SAFETY: 强制转换生命周期
                Ok(unsafe { transmute(rs) })
            },
            Err(e) => {
                if matches!(e, Error::QueryTimeout) {
                    warn!(
                        "#{}> [{}] @{} - elapsed: {}ms, timeout, sql: {}",
                        conn.spid(),
                        conn.log_category(),
                        conn.log_db_name(),
                        now.elapsed().as_millis(),
                        sql_preview
                    );
                    //FIXME
                    //自动重连
                    conn.reconnect().await?;
                } else {
                    conn.set_pending(false);
                    warn!(
                        "#{}> [{}] @{} - elapsed: {}ms, error: {}, sql: {}",
                        conn.spid(),
                        conn.log_category(),
                        conn.log_db_name(),
                        now.elapsed().as_millis(),
                        e,
                        sql_preview
                    );
                }
                Err(e)
            }
        }
    }
}

pub trait IntoSql<'a> {
    fn into_sql(self) -> Sql<'a>;
}

impl<'a> IntoSql<'a> for &'a str {
    fn into_sql(self) -> Sql<'a> { Sql::new(self) }
}

impl<'a> IntoSql<'a> for String {
    fn into_sql(self) -> Sql<'a> { Sql::new(self) }
}

impl<'a> IntoSql<'a> for Sql<'a> {
    fn into_sql(self) -> Sql<'a> { self }
}
