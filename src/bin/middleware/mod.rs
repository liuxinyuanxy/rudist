#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        tracing::debug!("Received request {:?}", &req);
        let resp = self.0.call(cx, req).await;
        tracing::debug!("Sent response {:?}", &resp);
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}

#[derive(Clone)]
pub struct CheckService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for CheckService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
    anyhow::Error: Into<S::Error>,
{
    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let req_str = format!("{:?}", &req);
        let forbidden = [
            "shit",
            "piss",
            "fuck",
            "cunt",
            "cocksucker",
            "motherfucker",
            "tits",
        ];
        if forbidden.iter().any(|&x| req_str.contains(x)) {
            Err(anyhow::anyhow!("Using dirty words is not allowed!").into())
        } else {
            self.0.call(cx, req).await
        }
    }
}

pub struct CheckLayer;

impl<S> volo::Layer<S> for CheckLayer {
    type Service = CheckService<S>;

    fn layer(self, inner: S) -> Self::Service {
        CheckService(inner)
    }
}
