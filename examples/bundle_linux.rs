use std::fs;
use std::path::PathBuf;

fn main() {
    let exe_bytes = fs::read("target/x86_64-unknown-linux-musl/release/library_app")
        .expect("Failed to read linux binary");
    let out_dir = PathBuf::from("target/x86_64-unknown-linux-musl/release/assets");
    let cache_dir = PathBuf::from("target/topcoat/cache/assets");

    fs::create_dir_all(&out_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let config = topcoat_asset::BundlerConfig::new().cache_dir(cache_dir);
    topcoat_asset::Bundler::new(&config)
        .bundle(&exe_bytes, &out_dir)
        .expect("Failed to bundle assets from linux binary");

    println!("Successfully bundled assets from Linux binary into target/x86_64-unknown-linux-musl/release/assets");
}
