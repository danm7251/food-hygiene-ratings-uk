use worker::*;

#[event(fetch)]
async fn fetch(
    _req: Request,
    _env: Env,
    _ctx: Context,
) -> Result<Response> {
    let router = Router::new();

    router.get_async("/search", |_req, _ctx| async move {
        Response::ok("Hello world!")
    }).run(_req, _env).await
}