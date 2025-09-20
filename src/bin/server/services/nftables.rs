use std::{fs, path::PathBuf};

pub fn apply_nft() {
    let src = PathBuf::from("assets/proxy.nft");

    let out_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = PathBuf::from(out_dir).join("target").join("assets");

    fs::create_dir_all(&dst).unwrap();
    fs::copy(&src, dst.join("proxy.nft")).unwrap();
}
