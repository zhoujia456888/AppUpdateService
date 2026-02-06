use crate::model::response::{ApiResponse, JSON_WRITTEN_KEY};
use salvo::oapi::endpoint::EndpointOutRegister;
use salvo::oapi::{ToResponse, ToSchema};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NoData {}

#[derive(Debug, Error)]
pub enum AppError {
    ///请求体错误
    #[error("{0}")]
    BadRequest(String),
    ///404未找到
    #[error("{0}")]
    NotFound(String),
    ///不可处理
    #[error("{0}")]
    Unprocessable(String),
    ///服务器内部错误
    #[error("{0}")]
    Internal(String),
    ///未授权
    #[error("{0}")]
    UnAuthorized(String),
    ///禁止
    #[error("{0}")]
    FORBIDDEN(String),
}

impl AppError {
    pub fn http_status(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unprocessable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::UnAuthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::FORBIDDEN(_) => StatusCode::FORBIDDEN,
        }
    }

    pub fn to_body(&self) -> ApiResponse<NoData> {
        ApiResponse {
            data: None,
            code: self.http_status().as_u16(),
            msg: self.to_string(),
        }
    }
}

#[async_trait]
impl Writer for AppError {
    async fn write(self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        depot.insert(JSON_WRITTEN_KEY, true);

        res.status_code(self.http_status());
        Json(self.to_body()).write(req, depot, res).await;
    }
}

/// 仅用于 OpenAPI 文档：错误响应 body（schema = ApiResponse<NoData>）
#[derive(ToResponse)]
#[salvo(response(description = "Error response", content_type = "application/json"))]
pub struct ErrorResponseBody;

/// ✅ handler 的唯一返回类型：不需要 Ok/Err，不需要 .into()
pub enum ApiOut<T: salvo_oapi::ToSchema> {
    Ok(ApiResponse<T>),
    Err(AppError),
}

impl<T> ApiOut<T>
where
    T: ToSchema,
{
    pub fn ok(data: T) -> Self {
        ApiOut::Ok(ApiResponse::ok(data))
    }

    pub fn err(err: AppError) -> Self {
        ApiOut::Err(err)
    }
}

#[async_trait]
impl<T> Writer for ApiOut<T>
where
    T: Serialize + ToSchema + Send + Sync + 'static,
{
    async fn write(self, req: &mut Request, depot: &mut Depot, res: &mut Response) {
        match self {
            ApiOut::Ok(ok) => ok.write(req, depot, res).await,
            ApiOut::Err(err) => err.write(req, depot, res).await,
        }
    }
}

/// OpenAPI：ApiOut<T> 自动挂 200 + 常见错误码
impl<T> EndpointOutRegister for ApiOut<T>
where
    T: ToSchema + 'static,
{
    fn register(components: &mut salvo::oapi::Components, operation: &mut salvo::oapi::Operation) {
        // 200 成功
        <ApiResponse<T> as EndpointOutRegister>::register(components, operation);

        // 4xx/5xx 错误（同一个 schema，挂多个状态码）
        let err_ref = <ErrorResponseBody as ToResponse>::to_response(components);

        let mut op = std::mem::take(operation);
        for sc in [
            StatusCode::BAD_REQUEST,
            StatusCode::NOT_FOUND,
            StatusCode::UNPROCESSABLE_ENTITY,
            StatusCode::INTERNAL_SERVER_ERROR,

        ] {
            op = op.add_response(sc.as_str(), err_ref.clone());
        }
        *operation = op;
    }
}
