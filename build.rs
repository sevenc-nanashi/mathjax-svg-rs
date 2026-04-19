fn main() {
    println!("cargo:rerun-if-changed=./js/dist/index.js");
}
