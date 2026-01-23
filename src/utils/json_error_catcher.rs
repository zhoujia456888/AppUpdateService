use salvo::{handler, Depot, FlowCtrl, Response};
use salvo::http::StatusCode;
use salvo::prelude::Json;
use crate::model::error::NoData;
use crate::model::response::{ApiResponse, JSON_WRITTEN_KEY};

#[handler]
pub async fn json_error_catcher(depot: &mut Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    let Some(status) = res.status_code else {
        return;
    };
    if !status.is_client_error() && !status.is_server_error() {
        return;
    }

    if depot.get::<bool>(JSON_WRITTEN_KEY).is_ok() {
        return;
    }

    let msg = match status {
        StatusCode::BAD_REQUEST => "bad request",
        StatusCode::NOT_FOUND => "not found",
        StatusCode::METHOD_NOT_ALLOWED => "method not allowed",
        StatusCode::UNPROCESSABLE_ENTITY => "unprocessable entity",
        _ => "internal server error",
    };

    res.render(Json(ApiResponse::<NoData> {
        data: None,
        code: status.as_u16(),
        msg: msg.to_string(),
    }));

    ctrl.skip_rest();
}