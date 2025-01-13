#[cfg(windows)]
extern crate windres;

fn main() -> std::io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winresource::WindowsResource::new();
        let name = env!("CARGO_PKG_NAME");
        let vers = env!("CARGO_PKG_VERSION");

        let mut sv = vers.split('.').collect::<Vec<_>>();
        while sv.len() < 4 {
            sv.push("0");
        }
        let file_version = format!("{}, {}, {}, {}", sv[0], sv[1], sv[2], sv[3]);

        windres::Build::new()
            .define("THE_PROJECT", Some(format!(r#""{name}"#).as_str()))
            .define("THE_VERSION", Some(format!(r#""{vers}"#).as_str()))
            .define("THE_FILEVESION", Some(file_version.as_str()))
            .compile("res/resource.rc")?;

        for entry in std::fs::read_dir("res")? {
            let entry = entry?;
            println!("cargo:retrun-if-changed={}", entry.path().display());
        }
    }

    Ok(())
}
