use crate::model::body::parse_json_body;
use crate::model::error::{ApiOut, AppError};
use crate::model::users::{GetUserReq, GetUserResp};
use salvo::prelude::*;

#[endpoint(tags("ping"), summary = "ping测试", description = "ping测试")]
pub async fn ping() -> ApiOut<String> {
    ApiOut::ok("ping success! 可以ping通！".to_string()).into()
}

#[endpoint(tags("ping"), summary = "bad_test 错误测试",  request_body = GetUserReq,description = "bad_test,测试报错")]
pub async fn bad_test(req: &mut Request) -> ApiOut<String> {
    let input = match parse_json_body::<GetUserReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    if input.id <= 0 {
        return ApiOut::err(AppError::BadRequest("id must be positive".into()));
    }
    if input.id != 1 {
        return ApiOut::err(AppError::NotFound(format!("user {} not found", input.id)));
    }
    ApiOut::ok("Desmond".to_string())
}

#[endpoint(tags("ping"), summary = "获取用户测试", request_body = GetUserReq)]
pub async fn resp_test(req: &mut Request) -> ApiOut<GetUserResp> {
    let input = match parse_json_body::<GetUserReq>(req).await {
        Ok(v) => v,
        Err(e) => return ApiOut::err(e),
    };

    if input.id <= 0 {
        return ApiOut::err(AppError::BadRequest("id must be positive".into()));
    }
    if input.id != 1 {
        return ApiOut::err(AppError::NotFound(format!("user {} not found", input.id)));
    }

    ApiOut::ok(GetUserResp {
        name: "Desmond".to_string(),
    })
}



pub fn ping_router() -> Router {
    Router::with_path("ping")
        .push(Router::with_path("ping").get(ping))
        .push(Router::with_path("bad_test").post(bad_test))
        .push(Router::with_path("resp_test").post(resp_test))
}
