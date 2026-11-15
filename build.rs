#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("app.manifest");
    let version = env!("CARGO_PKG_VERSION");
    let is_beta = version.contains("beta");
    let prod_name = if is_beta { "UniLauncher (BETA)" } else { "UniLauncher" };
    let desc = if is_beta {
        "UniLauncher - Lanceur universel de jeux PC (BETA)"
    } else {
        "UniLauncher - Lanceur universel de jeux PC"
    };
    res.set("FileDescription", desc);
    res.set("ProductName", prod_name);
    res.set("CompanyName", "nowxyii3");
    res.set("LegalCopyright", "Copyright (C) 2026 nowxyii3. Licence MIT.");
    res.set("InternalName", "UniLauncher");
    res.set("OriginalFilename", "UniLauncher.exe");
    let _ = res.compile();
}

#[cfg(not(windows))]
fn main() {}
