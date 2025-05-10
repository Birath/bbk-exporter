use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use bbk::Bbk;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::header::CONTENT_TYPE;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto;
use lazy_static::lazy_static;
use prometheus::{Encoder, GaugeVec, TextEncoder, register_gauge_vec};

pub mod bbk;

lazy_static! {
    static ref DOWNLOAD_SPEED_GAUGE_VEC: GaugeVec = register_gauge_vec!(
        "bbk_download_speed_mbps",
        "Download speed in Mbit/s",
        &["server", "network_operator"]
    )
    .unwrap();
    static ref UPLOAD_SPEED_GAUGE_VEC: GaugeVec = register_gauge_vec!(
        "bbk_upload_speed_mbps",
        "Upload speed in Mbit/s",
        &["server", "network_operator"]
    )
    .unwrap();
    static ref PING_GAUGE_VEC: GaugeVec = register_gauge_vec!(
        "bbk_latency_ms",
        "Latency to test server in ms",
        &["server", "network_operator"]
    )
    .unwrap();
    static ref BBK_PATH: PathBuf = PathBuf::from("bbk");
}

#[derive(Debug, Clone)]
struct ExporterContext {
    bbk_config: Bbk,
}

async fn metrics(
    context: ExporterContext,
    _: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    let encoder = TextEncoder::new();

    let bbk_output = match context.bbk_config.run_bbk() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to run BBK: {:?}", e);
            return Ok(Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(full("Failure while running bbk").boxed())
                .unwrap());
        }
    };

    DOWNLOAD_SPEED_GAUGE_VEC
        .with_label_values(&[bbk_output.server.clone(), bbk_output.isp.clone()])
        .set(bbk_output.download);
    UPLOAD_SPEED_GAUGE_VEC
        .with_label_values(&[bbk_output.server.clone(), bbk_output.isp.clone()])
        .set(bbk_output.upload);
    PING_GAUGE_VEC
        .with_label_values(&[bbk_output.server.clone(), bbk_output.isp.clone()])
        .set(bbk_output.ping);

    let metrics = prometheus::gather();
    let body = match encoder.encode_to_string(&metrics) {
        Ok(b) => b,
        Err(_) => return Ok(internal_server_error()),
    };
    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(full(body).boxed())
        .expect("Hello");
    Ok(response)
}

async fn handle(
    context: ExporterContext,
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => Ok(metrics(context, req).await.unwrap()),
        _ => {
            let not_found = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full("Not found"))
                .unwrap();
            Ok(not_found)
        }
    }
}

fn internal_server_error() -> Response<BoxBody<Bytes, hyper::Error>> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(full("Failure while running bbk").boxed())
        .unwrap()
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> hyper::rt::Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

pub async fn run_exporter(
    serving_port: u16,
    bbk: PathBuf,
    bbk_args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting BBK exporter on port {}", serving_port);
    let context = Arc::new(ExporterContext {
        bbk_config: Bbk {
            path: bbk,
            args: bbk_args,
        },
    });
    let addr = SocketAddr::from(([0, 0, 0, 0], serving_port));

    let listener = tokio::net::TcpListener::bind(addr).await?;

    loop {
        let context = context.clone();
        let service = service_fn(move |req| handle(context.as_ref().to_owned(), req));
        let (stream, _) = listener.accept().await?;

        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = auto::Builder::new(TokioExecutor)
                .serve_connection(io, service)
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
