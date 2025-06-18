use std::process::Command;
use std::io::stdin;

pub fn uninstall() {
    println!("Uninstalling cortexflow...");
    let mut userinput: String = String::new();
    println!("Select one option");
    display_uninstall_options();
    stdin().read_line(&mut userinput).expect("Error reading user input");

    if userinput == "1" {
        println!("Deleting cortexflow components");

        Command::new("kubectl")
            .args(["delete", "namespace", "cortexflow"])
            .output()
            .expect("Error deleting cortexflow namespace");
    } else if userinput == "2" {
        println!("Deleting cortexflow-identity service");

        Command::new("kubectl")
            .args(["delete", "daemonset", "cortexflow-identity"])
            .output()
            .expect("Error deleting cortexflow-identity");
    }
}
fn display_uninstall_options() {
    println!("1 > all");
    println!("2 > identity-service");
}
