use crate::schema::auth_captcha;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Serialize, Deserialize, Debug, Clone, Selectable)]
#[diesel(table_name = auth_captcha)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(sql_type=Timestamp)]
pub struct AuthCaptchaRecord {
    pub captcha_id: String,
    pub captcha_text: String,
    pub create_time: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}
