/// 数据库类型提取 trait
///
/// 用于从 Pool 或 Transaction 中自动推断数据库类型
/// 这样用户就不需要显式指定 `find_by_id::<sqlx::MySql, _>`，而是可以直接调用 `find_by_id_auto(pool, id)`
pub trait DatabaseType {
    /// 关联的数据库类型
    type DB: sqlx::Database + crate::database_info::DatabaseInfo;
}

// 为 Pool 实现 DatabaseType
#[cfg(feature = "mysql")]
impl DatabaseType for sqlx::Pool<sqlx::MySql> {
    type DB = sqlx::MySql;
}

#[cfg(feature = "postgres")]
impl DatabaseType for sqlx::Pool<sqlx::Postgres> {
    type DB = sqlx::Postgres;
}

#[cfg(feature = "sqlite")]
impl DatabaseType for sqlx::Pool<sqlx::Sqlite> {
    type DB = sqlx::Sqlite;
}

// 为 &Pool 实现 DatabaseType
#[cfg(feature = "mysql")]
impl DatabaseType for &sqlx::Pool<sqlx::MySql> {
    type DB = sqlx::MySql;
}

#[cfg(feature = "postgres")]
impl DatabaseType for &sqlx::Pool<sqlx::Postgres> {
    type DB = sqlx::Postgres;
}

#[cfg(feature = "sqlite")]
impl DatabaseType for &sqlx::Pool<sqlx::Sqlite> {
    type DB = sqlx::Sqlite;
}

// 为 Transaction 实现 DatabaseType
#[cfg(feature = "mysql")]
impl<'tx> DatabaseType for &mut sqlx::Transaction<'tx, sqlx::MySql> {
    type DB = sqlx::MySql;
}

#[cfg(feature = "postgres")]
impl<'tx> DatabaseType for &mut sqlx::Transaction<'tx, sqlx::Postgres> {
    type DB = sqlx::Postgres;
}

#[cfg(feature = "sqlite")]
impl<'tx> DatabaseType for &mut sqlx::Transaction<'tx, sqlx::Sqlite> {
    type DB = sqlx::Sqlite;
}

// 为 Connection 类型实现 DatabaseType（as_mysql_executor 等返回的是 Connection）
#[cfg(feature = "mysql")]
impl DatabaseType for sqlx::MySqlConnection {
    type DB = sqlx::MySql;
}

#[cfg(feature = "mysql")]
impl DatabaseType for &mut sqlx::MySqlConnection {
    type DB = sqlx::MySql;
}

#[cfg(feature = "postgres")]
impl DatabaseType for sqlx::PgConnection {
    type DB = sqlx::Postgres;
}

#[cfg(feature = "postgres")]
impl DatabaseType for &mut sqlx::PgConnection {
    type DB = sqlx::Postgres;
}

#[cfg(feature = "sqlite")]
impl DatabaseType for sqlx::SqliteConnection {
    type DB = sqlx::Sqlite;
}

#[cfg(feature = "sqlite")]
impl DatabaseType for &mut sqlx::SqliteConnection {
    type DB = sqlx::Sqlite;
}
