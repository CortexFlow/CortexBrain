use std::process::Command;

pub fn install_cortexflow() {
    println!("Preparing cortexflow installation");

    /* copying the files */
    println!("🛸 Copying kubernetes manifests as temporary files");
    copy_installation_files();

    println!("🛸 creating cortexflow namespace");
    /* create the namespace */
    Command::new("kubectl")
        .args(["create", "namespace", "cortexflow"])
        .output()
        .expect("Failed to create cortexflow namespace");

    /* install cert manager */
    println!("🛸 installing cert manager");
    println!("\n");
    Command::new("kubectl").args([
        "apply",
        "-f",
        "https://github.com/cert-manager/cert-manager/releases/latest/download/cert-manager.yaml",
    ]);
    Command::new("sleep").args(["110"]);
    println!("\n");

    /* install the components */
    println!("🛸 installing cortexflow components");
    install_components();
    println!("\n");

    println!("setupping cert manager");
    setup_cert_manager();
    println!("🛸 Removing temporary files");
    /* remove all the installation files */
    rm_installation_files();
    println!("🛸 installation completed");
}

fn install_components() {
    Command::new("kubectl").args(["apply", "-f", "configmap.yaml", "-n", "cortexflow"]);
    Command::new("kubectl").args(["apply", "-f", "configmap-role.yaml", "-n", "default"]);
    Command::new("kubectl").args(["apply", "-f", "rolebinding.yaml", "-n", "kube-system"]);
    Command::new("kubectl").args(["apply", "-f", "cortexflow-rolebinding.yaml"]);
    Command::new("kubectl").args(["apply", "-f", "certificate-manager.yaml"]);
}
fn copy_installation_files() {
    Command::new("cp")
        .args(["../../core/src/testing/configmap.yaml", "configmap.yaml"])
        .output()
        .expect("cannot import configmap file");
    Command::new("cp")
        .args(["../../core/src/testing/configmap-role.yaml", "configmap-role.yaml"])
        .output()
        .expect("cannot import configmap-role file");
    Command::new("cp")
        .args(["../../core/src/testing/rolebinding.yaml", "rolebinding.yaml"])
        .output()
        .expect("cannot import rolebinding file");
    Command::new("cp")
        .args(["../../core/src/testing/cortexflow-rolebinding.yaml", "cortexflow-rolebinding.yaml"])
        .output()
        .expect("cannot import rolebinding file");
    Command::new("cp")
        .args(["../../core/src/testing/certificate-manager.yaml", "certificate-manager.yaml"])
        .output()
        .expect("cannot import certificate-manager file");
}
fn rm_installation_files() {
    Command::new("rm")
        .args(["-rf", "configmap.yaml"])
        .output()
        .expect("cannot remove configmap file");
    Command::new("rm")
        .args(["-rf", "configmap-role.yaml"])
        .output()
        .expect("cannot remove configmap-role file");
    Command::new("rm")
        .args(["-rf", "rolebinding.yaml"])
        .output()
        .expect("cannot remove rolebinding file");
    Command::new("rm")
        .args(["-rf", "cortexflow-rolebinding.yaml"])
        .output()
        .expect("cannot remove rolebinding file");
    Command::new("rm")
        .args(["-rf", "certificate-manager.yaml"])
        .output()
        .expect("cannot remove certificate-manager file");
}
fn setup_cert_manager() {
    println!("🛸 creating the issuer");
    println!("🛸 caBundle certificate");
    println!("🛸 tls.key code");
    println!("🛸 tls.cert code");
}
