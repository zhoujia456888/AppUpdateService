use std::time::Instant;
use salvo::prelude::*;
use salvo::http::StatusCode;

#[handler]
pub async fn access_log(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
    ctrl: &mut FlowCtrl,
) {
    let start = Instant::now();
    ctrl.call_next(req, depot, res).await;

    let ms = start.elapsed().as_millis();
    let status = res.status_code.unwrap_or(StatusCode::NOT_FOUND).as_u16();

    tracing::info!(
        target: "access",
        "{} {} {} {} {}ms",
        req.remote_addr(),
        req.method(),
        req.uri().path(),
        status,
        ms
    );
}
