use crate::db::DbPool;
use crate::model::captcha::AuthCaptchaRecord;
use crate::model::error::AppError;
use crate::schema::auth_captcha;
use chrono::{Duration, Local};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use salvo::prelude::async_trait;
use std::sync::Arc;

const CAPTCHA_TTL_MINUTES: i64 = 10;

#[async_trait]
pub trait CaptchaStore: Send + Sync {
    async fn insert(&self, captcha_id: String, captcha_text: String) -> Result<(), AppError>;
    async fn get(&self, captcha_id: &str) -> Result<Option<String>, AppError>;
    async fn invalidate(&self, captcha_id: &str) -> Result<(), AppError>;
}

pub struct PostgresCaptchaStore {
    pool: Arc<DbPool>,
}

impl PostgresCaptchaStore {
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

    fn cleanup_expired(&self, conn: &mut PgConnection) -> Result<(), AppError> {
        diesel::delete(auth_captcha::table.filter(auth_captcha::expires_at.le(Local::now().naive_local())))
            .execute(conn)
            .map(|_| ())
            .map_err(|e| AppError::Internal(format!("清理过期验证码失败: {}", e)))
    }
}

#[async_trait]
impl CaptchaStore for PostgresCaptchaStore {
    async fn insert(&self, captcha_id: String, captcha_text: String) -> Result<(), AppError> {
        let mut conn = self.get_connection()?;
        self.cleanup_expired(&mut conn)?;

        let now = Local::now().naive_local();
        let new_record = AuthCaptchaRecord {
            captcha_id,
            captcha_text,
            create_time: now,
            expires_at: now + Duration::minutes(CAPTCHA_TTL_MINUTES),
        };

        diesel::insert_into(auth_captcha::table)
            .values(&new_record)
            .on_conflict(auth_captcha::captcha_id)
            .do_update()
            .set((
                auth_captcha::captcha_text.eq(&new_record.captcha_text),
                auth_captcha::create_time.eq(new_record.create_time),
                auth_captcha::expires_at.eq(new_record.expires_at),
            ))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(|e| AppError::Internal(format!("保存验证码失败: {}", e)))
    }

    async fn get(&self, captcha_id: &str) -> Result<Option<String>, AppError> {
        let mut conn = self.get_connection()?;
        self.cleanup_expired(&mut conn)?;

        auth_captcha::table
            .filter(auth_captcha::captcha_id.eq(captcha_id))
            .filter(auth_captcha::expires_at.gt(Local::now().naive_local()))
            .select(AuthCaptchaRecord::as_select())
            .first::<AuthCaptchaRecord>(&mut conn)
            .optional()
            .map(|record| record.map(|value| value.captcha_text))
            .map_err(|e| AppError::Internal(format!("查询验证码失败: {}", e)))
    }

    async fn invalidate(&self, captcha_id: &str) -> Result<(), AppError> {
        let mut conn = self.get_connection()?;

        diesel::delete(auth_captcha::table.filter(auth_captcha::captcha_id.eq(captcha_id)))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(|e| AppError::Internal(format!("删除验证码失败: {}", e)))
    }
}
