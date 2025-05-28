pub fn install_cortexflow() {
    println!("Preparing cortexflow installation");
    println!("🛸 Copying kubernetes manifests as temporary files");
    /* copying the files */

    println!("🛸 creating cortexflow namespace");
    /* create the namespace */
    println!("🛸 installing cert manager");
    /* install cert manager */
    println!("🛸 installing cortexflow components");
    /* install the components */
    println!("🛸 creating the issuer");
    println!("🛸 caBundle certificate");
    println!("🛸 tls.key code");
    println!("🛸 tls.cert code");
    /* return the codes and automatically isntall the proxy injector */
    println!("🛸 Removing temporary files");
    /* remove all the installation files */
    println!("🛸 installation completed");
}
