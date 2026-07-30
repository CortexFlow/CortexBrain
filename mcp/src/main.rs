mod prometheus;
mod tools;

use crate::tools::PrometheusTool;
use anyhow::Result;
use rmcp::ServiceExt;
use tokio::io::{BufReader, BufWriter};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting cortexflow-mcp");

    let stdin = BufReader::new(tokio::io::stdin());
    let stdout = BufWriter::new(tokio::io::stdout());

    let service = PrometheusTool::new()?.serve((stdin, stdout)).await?;
    service.waiting().await?;

    eprintln!("cortexflow-mcp running");
    Ok(())
}
