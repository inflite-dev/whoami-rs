use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, Request};
use axum::routing::{Router, get, post};
use bytes::Bytes;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::{env, fmt};
use whoami as whoami_lib;

struct AppData {
    name: Option<String>,
    hostname: Option<String>,
    ips: Vec<IpAddr>,
}

impl fmt::Display for AppData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        if let Some(name) = &self.name {
            s += format!("Name: {name}\n").as_str()
        };
        if let Some(hostname) = &self.hostname {
            s += format!("Hostname: {hostname}\n",).as_str()
        };
        if !self.ips.is_empty() {
            for ip in &self.ips {
                let ip_str = match ip {
                    IpAddr::V4(ip4) => format!("{ip4}"),
                    IpAddr::V6(ip6) => format!("{ip6}"),
                };
                s += format!("IP: {ip_str}\n").as_str()
            }
        }
        write!(f, "{s}")
    }
}

fn get_app_data() -> AppData {
    let app_name = env::var_os("WHOAMI_NAME").and_then(|os| os.into_string().ok());
    let app_hostname = whoami_lib::hostname().ok();
    let app_ips = pnet::datalink::interfaces()
        .iter()
        .flat_map(|iface| iface.ips.iter().map(|ipnet| ipnet.ip()))
        .collect();

    AppData {
        name: app_name,
        hostname: app_hostname,
        ips: app_ips,
    }
}

async fn bench(State(bench_data): State<Bytes>) -> Bytes {
    bench_data
}

async fn hello() -> Bytes {
    // No exclamation or capitalization here for our hello world here;
    // the canonical version of this program is not so excited to be alive.
    // https://en.wikipedia.org/wiki/%22Hello,_World!%22_program
    Bytes::from("hello, world")
}

#[axum::debug_handler]
async fn whoami(State(app_data): State<Arc<AppData>>) -> Bytes {
    Bytes::from(format!("{app_data}"))
}

async fn echo_stream(request: Request<Body>) -> (HeaderMap, Body) {
    (
        // StatusCode::OK,
        HeaderMap::from_iter([(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        )]),
        Body::from_stream(request.into_body().into_data_stream()),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Demonstrate isolating routers with different state.

    // First state type: AppData.
    // Must implement Clone - can use Arc instead of deriving or implementing Clone.
    let app_data = Arc::new(get_app_data());
    let whoami_router = Router::new().route("/", get(whoami)).with_state(app_data);

    // Second state type: Bytes.
    // Already implements Clone.
    let bench_data = Bytes::from("1");
    let bench_router = Router::new()
        .route("/bench", get(bench).post(bench).put(bench))
        .with_state(bench_data);

    let app = Router::new()
        // Merge routes from whoami_router with only AppData state dependency.
        .merge(whoami_router)
        // Merge routes from bench_router with only Bytes state dependency;
        // merge will not propagate the state dependency up to previous routes.
        .merge(bench_router)
        // Routes without state
        .route("/hello", get(hello))
        .route("/echo", post(echo_stream).put(echo_stream));

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);

    if let Err(err) = server.await {
        eprintln!("server error: {err}");
    }
    Ok(())
}
