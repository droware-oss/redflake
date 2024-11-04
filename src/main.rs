use clap::Parser;
use redflake::snowflake::MACHINE;
use redflake::Handler;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal::ctrl_c;
use tokio::sync::{mpsc, watch, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn};
use tracing_subscriber::fmt;

#[derive(Parser)]
#[command(version)]
struct CmdArgs {
    /// Server port
    #[arg(short, long, default_value_t = 6380)]
    port: u16,
    /// Machine ID
    #[arg(short, long, default_value_t = 0)]
    machine: u8,
    /// Maximum concurrent connected clients
    #[arg(long, default_value_t = 1024)]
    max_clients: usize,
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // parse command-line arguments
    let args = CmdArgs::parse();
    MACHINE.set(args.machine).expect("Unable to set machine id");

    // setup global trace data collector
    let subscriber = fmt().with_target(false).with_thread_ids(true).finish();
    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, args.port))
        .await
        .expect("Unable to bind socket");

    info!(
        "Machine ID({}) started to listen on {}",
        args.machine, args.port
    );

    let semaphore = Arc::new(Semaphore::new(args.max_clients));
    let (closing_tx, _) = watch::channel(());
    let (conn_closed_tx, mut all_conn_closed_rx) = mpsc::unbounded_channel::<()>();

    tokio::select! {
        // handle connection
        _ = async {
            loop {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let (socket, addr) = listener.accept().await.expect("Failed to accept socket");
                let mut handler = Handler::new(socket, addr, closing_tx.subscribe(), conn_closed_tx.clone());
                tokio::spawn(async move {
                    if let Err(err) = handler.handle().await {
                        error!(cause = ?err, "Error while processing connection");
                    }
                    drop(permit);
                });
            }
        } => {},
        // handle shutdown
        _ = ctrl_c() => {}
    }

    // send close signal to all connections
    drop(closing_tx);

    // wait until all connections closed
    drop(conn_closed_tx);
    if let Err(_) = timeout(Duration::from_secs(10), all_conn_closed_rx.recv()).await {
        warn!("Forcing exit due to timeout");
    }

    info!("Machine ID({}) stopped", args.machine);
}
