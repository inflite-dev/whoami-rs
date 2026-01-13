use axum::extract::State;
use axum::routing::{Router, get};
use bytes::Bytes;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::{env, fmt};

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
    let app_hostname = whoami::hostname().ok();
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

async fn bench() -> Bytes {
    Bytes::from("1")
}

async fn hello() -> Bytes {
    // No exclamation or capitalization here for our hello world here;
    // the canonical version of this program is not so excited to be alive.
    // https://en.wikipedia.org/wiki/%22Hello,_World!%22_program
    Bytes::from("hello, world")
}

#[axum::debug_handler]
async fn whoami(app_data: State<Arc<AppData>>) -> Bytes {
    let app_data_ref = app_data.as_ref();
    Bytes::from(format!("{app_data_ref}"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app_data = Arc::new(get_app_data());

    let app = Router::new()
        .route("/", get(whoami).post(whoami).put(whoami))
        .route("/bench", get(bench).post(bench).put(bench))
        .route("/hello", get(hello).post(hello).put(hello))
        .with_state(app_data);

    let addr = SocketAddr::from(([0, 0, 0, 0], 5000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = axum::serve(listener, app);

    if let Err(err) = server.await {
        eprintln!("server error: {err}");
    }
    Ok(())
}
