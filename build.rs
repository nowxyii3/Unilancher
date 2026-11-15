#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("app.manifest");
    res.set("FileDescription", "UniLauncher - Lanceur universel de jeux PC (BETA)");
    res.set("ProductName", "UniLauncher (BETA)");
    res.set("CompanyName", "nowxyii3");
    res.set("LegalCopyright", "Copyright (C) 2026 nowxyii3. Licence MIT.");
    res.set("InternalName", "UniLauncher");
    res.set("OriginalFilename", "UniLauncher.exe");
    let _ = res.compile();
}

#[cfg(not(windows))]
fn main() {}
