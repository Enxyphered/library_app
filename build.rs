fn main() {
    println!("cargo:rerun-if-changed=styles.css");
    topcoat::tailwind::BuildConfig::new()
        .input("styles.css")
        .render()
        .unwrap();
}
