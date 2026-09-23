use std::process::{Command, Output};

// docs:
//
// Abstraction over external process execution (kubectl, cargo, ...) so callers
// can inject a stub in tests instead of shelling out to a real binary/cluster.

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[String]) -> std::io::Result<Output>;
}

pub struct RealCommandRunner;

impl CommandRunner for RealCommandRunner {
    fn run(&self, program: &str, args: &[String]) -> std::io::Result<Output> {
        Command::new(program).args(args).output()
    }
}

#[cfg(test)]
pub mod test_support {
    use super::CommandRunner;
    use std::process::{ExitStatus, Output};

    #[cfg(unix)]
    fn exit_status(success: bool) -> ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(if success { 0 } else { 1 << 8 })
    }

    // Stubs process output so tests never shell out to a real binary or cluster.
    pub struct StubCommandRunner {
        result: std::io::Result<Output>,
    }

    impl StubCommandRunner {
        pub fn success(stdout: &str) -> Self {
            Self {
                result: Ok(Output {
                    status: exit_status(true),
                    stdout: stdout.as_bytes().to_vec(),
                    stderr: Vec::new(),
                }),
            }
        }

        pub fn failure(stderr: &str) -> Self {
            Self {
                result: Ok(Output {
                    status: exit_status(false),
                    stdout: Vec::new(),
                    stderr: stderr.as_bytes().to_vec(),
                }),
            }
        }

        pub fn io_error() -> Self {
            Self {
                result: Err(std::io::Error::other("command not found")),
            }
        }
    }

    impl CommandRunner for StubCommandRunner {
        fn run(&self, _program: &str, _args: &[String]) -> std::io::Result<Output> {
            match &self.result {
                Ok(output) => Ok(Output {
                    status: output.status,
                    stdout: output.stdout.clone(),
                    stderr: output.stderr.clone(),
                }),
                Err(_) => Err(std::io::Error::other("command not found")),
            }
        }
    }
}
