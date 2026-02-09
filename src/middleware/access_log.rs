use salvo::{Depot, FlowCtrl, Handler, Request, Response};
use std::time::Instant;

pub struct AccessLog;

#[salvo::async_trait]
impl Handler for AccessLog {
    async fn handle(
        &self,
        req: &mut Request,
        depot: &mut Depot,
        res: &mut Response,
        ctrl: &mut FlowCtrl,
    ) {
        let start = Instant::now();
        ctrl.call_next(req, depot, res).await;

        let latency_ms = start.elapsed().as_millis();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        use salvo::http::StatusCode;

        let status = res.status_code.unwrap_or(StatusCode::OK).as_u16();

        // ✅ 关键：remote_addr 不是 Option/Iterator，直接 to_string 即可
        let ip = req.remote_addr().to_string();

        let ua = req
            .headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");

        tracing::info!(
            target: "access",
            %method,
            %path,
            status,
            latency_ms,
            %ip,
            user_agent = %ua,
        );
    }
}
