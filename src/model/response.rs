use salvo::oapi::ToSchema;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ApiResponse<T>
where
    T: ToSchema,
{
    pub data: Option<T>,
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

#[async_trait]
impl<T> Writer for ApiResponse<T>
where
    T: Serialize + ToSchema + Send + Sync,
{
    async fn write(self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        // 如果没设置状态码，默认 200
        if res.status_code.is_none() {
            res.status_code(StatusCode::OK);
        }
        Json(self).write(req, depot, res).await;
    }
}

impl<T> EndpointOutRegister for ApiResponse<T>
where
    T: ToSchema + 'static,
{
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        <Json<ApiResponse<T>> as EndpointOutRegister>::register(
            components,
            operation,
        );
    }
}
