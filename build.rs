fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/K1R4LABS.ico");
        res.set("FileDescription", "K1R4LABS Riot Account Switcher");
        res.set("ProductName", "K1R4LABS — Riot Account Switcher");
        res.set("CompanyName", "K1R4LABS");
        res.set("LegalCopyright", "© 2026 by Kira Kohler — All rights reserved.");
        res.set("FileVersion", "1.1.0.0");
        res.set("ProductVersion", "1.1.0");
        res.set_manifest(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false"/>
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>"#,
        );
        res.compile().unwrap();
    }
}
