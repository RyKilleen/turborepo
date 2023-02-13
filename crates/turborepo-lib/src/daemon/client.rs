use thiserror::Error;
use tonic::{Code, Status};

use self::proto::turbod_client::TurbodClient;
use super::connector::DaemonConnector;
use crate::get_version;

pub mod proto {
    tonic::include_proto!("turbodprotocol");
}

#[derive(Debug)]
pub struct DaemonClient<T> {
    pub client: TurbodClient<tonic::transport::Channel>,
    pub connect_settings: T,
}

impl<T> DaemonClient<T> {
    /// Get the status of the daemon.
    pub async fn status(&mut self) -> Result<proto::DaemonStatus, DaemonError> {
        self.client
            .status(proto::StatusRequest {})
            .await?
            .into_inner()
            .daemon_status
            .ok_or(DaemonError::MissingResponse)
    }

    /// Stops the daemon and closes the connection, returning
    /// the connection settings that were used to connect.
    pub async fn stop(mut self) -> Result<T, DaemonError> {
        self.client.shutdown(proto::ShutdownRequest {}).await?;
        Ok(self.connect_settings)
    }

    /// Interrogate the server for its version.
    pub(super) async fn handshake(&mut self) -> Result<(), DaemonError> {
        let _ret = self
            .client
            .hello(proto::HelloRequest {
                version: get_version().to_string(),
                // todo(arlyon): add session id
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_changed_outputs(
        &mut self,
        hash: String,
        output_globs: Vec<String>,
    ) -> Result<Vec<String>, DaemonError> {
        Ok(self
            .client
            .get_changed_outputs(proto::GetChangedOutputsRequest { hash, output_globs })
            .await?
            .into_inner()
            .changed_output_globs)
    }

    #[allow(dead_code)]
    pub async fn notify_outputs_written(
        &mut self,
        hash: String,
        output_globs: Vec<String>,
        output_exclusion_globs: Vec<String>,
    ) -> Result<(), DaemonError> {
        self.client
            .notify_outputs_written(proto::NotifyOutputsWrittenRequest {
                hash,
                output_globs,
                output_exclusion_globs,
            })
            .await?;

        Ok(())
    }
}

impl DaemonClient<()> {
    /// Augment the client with the connect settings, allowing it to be
    /// restarted.
    pub fn with_connect_settings(
        self,
        connect_settings: DaemonConnector,
    ) -> DaemonClient<DaemonConnector> {
        DaemonClient {
            client: self.client,
            connect_settings,
        }
    }
}

impl DaemonClient<DaemonConnector> {
    /// Stops the daemon, closes the connection, and opens a new connection.
    pub async fn restart(self) -> Result<DaemonClient<DaemonConnector>, DaemonError> {
        self.stop().await?.connect().await
    }
}

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Failed to connect to daemon")]
    Connection,
    #[error("Daemon version mismatch")]
    VersionMismatch,
    #[error("could not connect: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),
    #[error("could not fork")]
    Fork,
    #[error("could not connect: {0}")]
    GrpcFailure(tonic::Code),
    #[error("missing response")]
    MissingResponse,
    #[error("could not read pid file")]
    PidFile,
    #[error("could not connect: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("daemon is not running and will not be started")]
    NotRunning,
}

impl From<Status> for DaemonError {
    fn from(status: Status) -> DaemonError {
        match status.code() {
            Code::FailedPrecondition => DaemonError::VersionMismatch,
            Code::Unavailable => DaemonError::Connection,
            c => DaemonError::GrpcFailure(c),
        }
    }
}