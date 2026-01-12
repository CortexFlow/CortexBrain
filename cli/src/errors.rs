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
    AgentError(tonic_reflection::server::Error),
    MonitoringError { reason: String },
}
// docs:
//
// The following functions implements the trait From conversions
//
// The From Trait is used to perform a value-to-value conversion while consuming input values.
// We use that to return a single error type 'CliError' that incapsulates multiple error types

impl From<kube::Error> for CliError {
    fn from(e: kube::Error) -> Self {
        CliError::ClientError(e)
    }
}
impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        CliError::MonitoringError {
            reason: e.to_string(),
        }
    }
}
impl From<prost::DecodeError> for CliError {
    fn from(e: prost::DecodeError) -> Self {
        return CliError::AgentError(tonic_reflection::server::Error::DecodeError(e));
    }
}
impl From<tonic::Status> for CliError {
    fn from(e: tonic::Status) -> Self {
        return CliError::MonitoringError {
            reason: e.to_string(),
        };
    }
}

// docs:
//
// The Trait fmt::Display is used to create a user friendly error message for the CliError type.
// This Trait automatically implements the ToString trait for the type allowing
// the usage of .to_string() method

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
                    reason.red().bold()
                )
            }
            CliError::MonitoringError { reason } => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".blue().bold(),
                    "An error occured while installing cortexflow components. Reason:"
                        .bold()
                        .red(),
                    reason.red().bold()
                )
            }
            CliError::ClientError(e) => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".blue().bold(),
                    "Client Error:".bold().red(),
                    e.to_string().red().bold()
                )
            }
            CliError::AgentError(e) => {
                write!(
                    f,
                    "{} {} {}",
                    "=====>".bold().blue(),
                    "Agent Error:".bold().red(),
                    e.to_string().bold().red()
                )
            }
        }
    }
}
