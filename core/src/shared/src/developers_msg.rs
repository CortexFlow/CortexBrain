/*
    Dashboard for CortexFlow Developers
    Includes:
        - Changelog
        - Whats new
        - Open Issues
*/

use tracing::{warn,info,instrument};

#[instrument]
pub fn info() {
    info!(
        "
        Welcome to the CortexFlow Developers Dashboard, introduced on January 27th, 2025.
        This tool provides a summary of updates, new features, and a list of unresolved issues in the core functionalities.
        The dashboard is designed to help CortexBrain developers focus on addressing key challenges in the core system, 
        enabling efficient collaboration and progress.
        \n"
    );
    warn!("Requirements: Docker, Kubernetes, Apache Kafka");
    whats_new();
    changelog();
    problems_to_solve();
}
#[instrument]
pub fn changelog() {
    info!("------------------ C H A N G E L O G -------------------\n");
    info!("29.01.2025");
    info!("1-added send message function and consume_and_forward functions in kafka.rs");
    info!("2-added expection handler in update_corefile function. If the interface is unavailable it show the available interfaces");
    info!("27.01.2025");
    info!("- Added APIs for 'Default' and 'V1' base configurations");
    info!("- Introduced a developer message tab");
    info!("- Refactored client code to align with the new crate structure");
    info!("- Added TODO comments for future improvements\n");
}

#[instrument]
pub fn whats_new() {
    warn!(
        "- This is the first pre-alpha version of CortexBrain. Expect some bugs as extensive testing is still required."
    );
}

#[instrument]
pub fn problems_to_solve() {
    warn!("--------------- O P E N   I S S U E S ------------------\n");
    warn!("1. The 'validation.rs' module requires full implementation.");
    warn!("2. The 'update_corefile' function requires a code review.");
    warn!("3. In 'edgecni.rs', the 'run' functionality needs implementation.");
    warn!("4. The 'stop' functionality in the 'close_route' function of 'edgecni.rs' needs implementation.\n");
}
