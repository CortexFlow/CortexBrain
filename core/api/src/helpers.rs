

use crate::constants::TASK_COMM_LEN;

pub fn comm_to_string(comm: &[u8; TASK_COMM_LEN]) -> String {
    let end = comm.iter().position(|&c| c == 0).unwrap_or(comm.len());
    String::from_utf8_lossy(&comm[..end]).to_string()
}
