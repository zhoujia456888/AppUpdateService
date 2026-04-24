use crate::db::DbPool;
use crate::model::error::AppError;
use crate::model::jwt::TokenResp;
use crate::model::users::User;
use crate::schema::users;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use salvo::prelude::async_trait;
use std::sync::Arc;
use uuid::Uuid;

enum TokenField {
    Access,
    Refresh,
}

#[async_trait]
pub trait TokenStore: Send + Sync {
    async fn save_tokens(
        &self,
        user_id: Uuid,
        access_token: String,
        refresh_token: String,
    ) -> Result<TokenResp, AppError>;

    async fn find_user_by_id_and_username(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<Option<User>, AppError>;

    async fn access_token_matches(
        &self,
        user_id: Uuid,
        access_token: &str,
    ) -> Result<bool, AppError>;

    async fn refresh_token_matches(
        &self,
        user_id: Uuid,
        refresh_token: &str,
    ) -> Result<bool, AppError>;
}

pub struct PostgresTokenStore {
    pool: Arc<DbPool>,
}

impl PostgresTokenStore {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn get_connection(
        &self,
    ) -> Result<PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::Internal(format!("数据库连接失败: {}", e)))
    }

    fn token_matches(&self, user_id: Uuid, token: &str, field: TokenField) -> Result<bool, AppError> {
        let mut conn = self.get_connection()?;

        let query = match field {
            TokenField::Access => users::table
                .filter(users::id.eq(user_id))
                .filter(users::access_token.eq(token))
                .into_boxed(),
            TokenField::Refresh => users::table
                .filter(users::id.eq(user_id))
                .filter(users::refresh_token.eq(token))
                .into_boxed(),
        };

        query
            .first::<User>(&mut conn)
            .optional()
            .map(|user| user.is_some())
            .map_err(|e| AppError::Internal(format!("数据库查询错误: {}", e)))
    }
}

#[async_trait]
impl TokenStore for PostgresTokenStore {
    async fn save_tokens(
        &self,
        user_id: Uuid,
        access_token: String,
        refresh_token: String,
    ) -> Result<TokenResp, AppError> {
        let mut conn = self.get_connection()?;

        let result = diesel::update(users::table.find(user_id))
            .set((
                users::access_token.eq(&access_token),
                users::refresh_token.eq(&refresh_token),
            ))
            .execute(&mut conn);

        match result {
            Ok(affected_rows) if affected_rows > 0 => Ok(TokenResp {
                access_token,
                refresh_token,
            }),
            Ok(_) => Err(AppError::Internal(format!(
                "未找到对应的user_id{}",
                user_id
            ))),
            Err(e) => Err(AppError::Internal(format!(
                "更新保存Token失败,请重试！'{}'",
                e
            ))),
        }
    }

    async fn find_user_by_id_and_username(
        &self,
        user_id: Uuid,
        username: &str,
    ) -> Result<Option<User>, AppError> {
        let mut conn = self.get_connection()?;

        users::table
            .filter(users::username.eq(username))
            .filter(users::id.eq(user_id))
            .first::<User>(&mut conn)
            .optional()
            .map_err(|e| AppError::Internal(format!("查询用户失败: {}", e)))
    }

    async fn access_token_matches(
        &self,
        user_id: Uuid,
        access_token: &str,
    ) -> Result<bool, AppError> {
        self.token_matches(user_id, access_token, TokenField::Access)
    }

    async fn refresh_token_matches(
        &self,
        user_id: Uuid,
        refresh_token: &str,
    ) -> Result<bool, AppError> {
        self.token_matches(user_id, refresh_token, TokenField::Refresh)
    }
}
