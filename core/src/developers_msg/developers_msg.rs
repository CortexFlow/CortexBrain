/*
    Dashboard for CortexFlow Developers
    Includes:
        - Changelog
        - Whats new
        - Open Issues
*/

pub fn info() {
    println!(
        "
        Welcome to the CortexFlow Developer Dashboard, introduced on January 27th, 2025.
        This tool provides a summary of updates, new features, and a list of unresolved issues in the core functionalities.
        The dashboard is designed to help CortexBrain developers focus on addressing key challenges in the core system, 
        enabling efficient collaboration and progress.
        \n"
    );
    changelog();
    whats_new();
    problems_to_solve();
}

pub fn changelog() {
    println!("------------------ C H A N G E L O G -------------------\n");
    println!("- Added APIs for 'Default' and 'V1' base configurations");
    println!("- Introduced a developer message tab");
    println!("- Refactored client code to align with the new crate structure");
    println!("- Added TODO comments for future improvements\n");
}

pub fn whats_new() {
    println!("---------------- W H A T ' S   N E W -------------------\n");
    println!(
        "- This is the first pre-alpha version of CortexBrain. Expect some bugs as extensive testing is still required."
    );
    println!(
        "- CortexBrain is an ambitious project aiming to connect cloud and edge computing in a fast, simple, and efficient way.\n"
    );
}

pub fn problems_to_solve() {
    println!("--------------- O P E N   I S S U E S ------------------\n");
    println!("1. In 'kernel.rs', the 'update_corefile' function needs an exception handler.");
    println!("2. The 'validation.rs' module requires full implementation.");
    println!("3. The 'update_corefile' function requires a code review.");
    println!("4. In 'edgecni.rs', the 'run' functionality needs implementation.");
    println!("5. The 'stop' functionality in the 'close_route' function of 'edgecni.rs' needs implementation.\n");
}
