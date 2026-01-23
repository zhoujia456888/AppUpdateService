use salvo::oapi::ToSchema;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

pub const JSON_WRITTEN_KEY: &str = "__json_written";

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T>
where
    T: ToSchema,
{
    pub data: Option<T>,
    /// 标准 code：直接使用 HTTP 状态码数字（200/400/404/422/500...）
    pub code: u16,
    pub msg: String,
}

impl<T> ApiResponse<T>
where
    T: ToSchema,
{
    pub fn ok(data: T) -> Self {
        Self {
            data: Some(data),
            code: StatusCode::OK.as_u16(),
            msg: "ok".to_string(),
        }
    }

    pub fn err(status: StatusCode, msg: impl Into<String>) -> Self {
        Self {
            data: None,
            code: status.as_u16(),
            msg: msg.into(),
        }
    }
}

/// 运行时：ApiResponse<T> 直接输出 JSON（handler 不需要外层 Json(...)）
#[async_trait]
impl<T> Writer for ApiResponse<T>
where
    T: serde::Serialize + salvo::oapi::ToSchema + Send + Sync,
{
    async fn write(
        self,
        req: &mut salvo::Request,
        depot: &mut salvo::Depot,
        res: &mut salvo::Response,
    ) {
        if self.code == 0 {
            // 什么都不做，不标记JSON_WRITTEN_KEY，让后续handler继续
            return;
        }

        // ✅ 标记：已经输出过我们自己的 JSON
        depot.insert(JSON_WRITTEN_KEY, true);

        if res.status_code.is_none() {
            res.status_code(StatusCode::OK);
        }
        Json(self).write(req, depot, res).await;
    }
}

/// OpenAPI：让 ApiResponse<T> 可作为 endpoint 输出类型注册
impl<T> EndpointOutRegister for ApiResponse<T>
where
    T: ToSchema + 'static,
{
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        <Json<ApiResponse<T>> as EndpointOutRegister>::register(components, operation);
    }
}
