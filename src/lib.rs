mod command;
mod connection;
mod frame;
pub mod snowflake;

use crate::command::Command;
use crate::connection::Connection;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;

#[derive(Debug)]
pub struct Handler {
    conn: Connection,
    closing: Receiver<()>,
    _closed: UnboundedSender<()>,
}

impl Handler {
    pub fn new(
        socket: TcpStream,
        addr: SocketAddr,
        closing: Receiver<()>,
        closed: UnboundedSender<()>,
    ) -> Handler {
        Handler {
            conn: Connection::new(socket, addr),
            closing,
            _closed: closed,
        }
    }

    pub async fn handle(&mut self) -> std::io::Result<()> {
        let mut shutting_down = false;
        let mut client_closed = false;
        while !shutting_down && !client_closed {
            if let Some(frame) = tokio::select! {
                frame = self.conn.read_frame() => frame?,
                _ = self.closing.changed() => { shutting_down = true; None },
            } {
                let cmd = Command::from_frame(frame)?;
                let resp = cmd.apply(&mut self.conn).await;
                self.conn.write_frame(&resp).await?;
            } else {
                client_closed = true;
            }
        }
        Ok(())
    }
}
