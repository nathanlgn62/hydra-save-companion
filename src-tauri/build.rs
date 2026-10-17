fn main() {
    println!("cargo:rerun-if-env-changed=STEAM_GRID_DB");
    println!("cargo:rerun-if-env-changed=HSC_GC_ID");
    println!("cargo:rerun-if-env-changed=HSC_GS");
    println!("cargo:rerun-if-env-changed=HSC_DBX_ID");
    println!("cargo:rerun-if-env-changed=HSC_DBX_SECRET");
    println!("cargo:rerun-if-env-changed=HSC_PROTON_ID");
    println!("cargo:rerun-if-env-changed=HSC_PROTON_SECRET");
    tauri_build::build()
}
