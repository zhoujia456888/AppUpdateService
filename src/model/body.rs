use serde::de::{DeserializeOwned, IntoDeserializer};
use serde_json::Value;
use crate::model::error::AppError;

/// 解析 JSON body，并返回字段级错误信息（如：`id: invalid type... expected i64`）
pub async fn parse_json_body<T>(req: &mut salvo::Request) -> Result<T, AppError>
where
    T: DeserializeOwned,
{
    // ① 先解析成 Value（JSON 语法错误会在这里返回）
    let value: Value = match req.parse_json::<Value>().await {
        Ok(v) => v,
        Err(e) => return Err(AppError::BadRequest(format!("invalid json: {e}"))),
    };

    // ② Value -> Deserializer（注意：不要 &mut value）
    let de = value.into_deserializer();

    // ③ 用 serde_path_to_error::deserialize 直接反序列化并拿路径
    match serde_path_to_error::deserialize::<_, T>(de) {
        Ok(v) => Ok(v),
        Err(e) => {
            let path = e.path().to_string();
            if path.is_empty() {
                Err(AppError::BadRequest(e.inner().to_string()))
            } else {
                Err(AppError::BadRequest(format!("{path}: {}", e.inner())))
            }
        }
    }
}
