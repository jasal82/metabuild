fn main() {
    println!("cargo:rerun-if-env-changed=TARGET");
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
}
