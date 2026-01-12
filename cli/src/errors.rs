use colored::Colorize;
use std::fmt;

// docs:
//
// CliError enum to group all the errors
//
// Custom error definition
// InstallerError:
//      - used for general installation errors occured during the installation of cortexflow components. Can be used for:
//          - Return downloading errors
//          - Return unsuccessful file removal during installation
//
// ClientError:
//      - used for Kubernetes client errors. Can be used for:
//          - Return client connection errors
//
// UninstallError:
//      - used for general installation errors occured during the uninstall for cortexflow components. Can be used for:
//          -  Return components removal errors
//
// AgentError:
//      - used for cortexflow agent errors. Can be used for:
//          - return errors from the reflection server
//          - return unavailable agent errors (404)
//
// MonitoringError:
//      - used for general monitoring errors. TODO: currently under implementation
//
// implements fmt::Display for user friendly error messages

#[derive(Debug)]
pub enum CliError {
    InstallerError { reason: String },
    ClientError(kube::Error),
    UninstallError { reason: String },
    AgentError(tonic_reflection::server::Error),
    MonitoringError { reason: String },
}
// docs:
// error type conversions

impl From<kube::Error> for CliError {
    fn from(e: kube::Error) -> Self {
        CliError::ClientError(e)
    }
}
impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::MonitoringError {
            reason: format!("{}", e),
        }
    }
}
impl From<()> for CliError {
    fn from(e: ()) -> Self {
        return ().into();
    }
}
impl From<prost::DecodeError> for CliError {
    fn from(e: prost::DecodeError) -> Self {
        todo!()
    }
}
impl From<tonic::Status> for CliError {
    fn from(e: tonic::Status) -> Self {
        todo!()
    }
}

// docs:
// fmt::Display implementation for CliError type. Creates a user friendly message error message.
// TODO: implement colored messages using the colorize crate for better output display

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InstallerError { reason } => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".blue().bold(),
                    "An error occured while installing cortexflow components. Reason:"
                        .bold()
                        .red(),
                    reason
                )
            }
            CliError::UninstallError { reason } => {
                write!(
                    f,
                    "An error occured while installing cortexflow components. Reason: {}",
                    reason
                )
            }
            CliError::MonitoringError { reason } => {
                write!(
                    f,
                    "An error occured while installing cortexflow components. Reason: {}",
                    reason
                )
            }
            CliError::ClientError(e) => write!(f, "Client Error: {}", e),
            CliError::AgentError(e) => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".bold().blue(),
                    "Agent Error:".bold().red(),
                    e
                )
            }
        }
    }
}
