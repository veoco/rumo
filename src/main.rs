use getopts::Options;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use std::net::SocketAddr;
use tokio::signal;

use rumo::{app, init};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} run|init [options]", program);
    print!("{}", opts.usage(&brief));
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "name", "set admin name", "NAME");
    opts.optopt("m", "mail", "set admin mail", "MAIL");
    opts.optopt("p", "password", "set admin password", "PASSWORD");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}\n", f.to_string());
            print_usage(&program, opts);
            return;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let command = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    match command.as_str() {
        "run" => {
            tracing_subscriber::fmt::init();
            let addr = env::var("LISTEN_ADDRESS").unwrap_or(String::from("127.0.0.1:3000"));
            info!("Listening on http://{}", addr);

            let app = app(None).await;
            let _ = axum::Server::bind(&addr.parse().unwrap())
                .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                .with_graceful_shutdown(shutdown_signal())
                .await;
        }
        "init" => {
            let subscriber = FmtSubscriber::builder()
                .with_max_level(Level::TRACE)
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("start log failed");

            let name = matches.opt_str("n").unwrap_or("admin".to_owned());
            let mail = matches.opt_str("m").unwrap_or("admin@local.host".to_owned());
            let password = matches.opt_str("p").unwrap_or("admin".to_owned());

            init(name, mail, password).await;
            info!("database created")
        }
        _ => {
            print_usage(&program, opts);
            return;
        }
    }
}
