use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let boot_dir = manifest.join("boot");

    if let Ok(entries) = fs::read_dir(&boot_dir) {
        for e in entries.flatten() {
            println!("cargo:rerun-if-changed={}", e.path().display());
        }
    }
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bootloader_src = boot_dir.join("bootloader.s");
    let kernel_src = boot_dir.join("kernel.s");

    let bootloader_bin = out_dir.join("bootloader.bin");
    let kernel_bin = out_dir.join("kernel.bin");

    assemble_link_bin(&bootloader_src, &bootloader_bin, "_start").expect("bootloader build");
    assemble_link_bin(&kernel_src, &kernel_bin, "_start").expect("kernel build");
}

fn assemble_link_bin(src: &Path, dst_bin: &Path, entry: &str) -> Result<(), String> {
    if !src.exists() {
        return Err(format!("source not found: {}", src.display()));
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let stem = src.file_stem().unwrap().to_string_lossy();
    let obj = out_dir.join(format!("{stem}.o"));
    let elf = out_dir.join(format!("{stem}.elf"));
    let lds = out_dir.join(format!("{stem}.ld"));

    // Minimal linker script: base at 0x0
    fs::File::create(&lds)
        .and_then(|mut f| {
            write!(
                f,
                r#"
ENTRY({entry})
SECTIONS {{
  . = 0x0;
  .text : {{ *(.text .text.*) }}
  .rodata : {{ *(.rodata .rodata.*) }}
  .data : {{ *(.data .data.*) }}
  .bss  : {{ *(.bss .bss.* COMMON) }}
}}
"#
            )
        })
        .map_err(|e| format!("write linker script: {e}"))?;

    let (as_cmd, ld_cmd, objcopy_cmd) = find_tools().ok_or_else(|| {
        "RISC-V binutils not found. Install riscv64-unknown-elf toolchain or set RISCV_PREFIX."
            .to_string()
    })?;

    // Assemble (no compressed)
    run(
        Command::new(&as_cmd)
            .arg("-march=rv64imafd")
            .arg("-mabi=lp64")
            .arg(src)
            .arg("-o")
            .arg(&obj),
        "as",
    )?;

    // Link at 0x0, no libs
    run(
        Command::new(&ld_cmd)
            .arg("-nostdlib")
            .arg("-static")
            .arg("-T")
            .arg(&lds)
            .arg("-o")
            .arg(&elf)
            .arg(&obj),
        "ld",
    )?;

    // Binary
    run(
        Command::new(&objcopy_cmd)
            .arg("-O")
            .arg("binary")
            .arg(&elf)
            .arg(dst_bin),
        "objcopy",
    )?;

    // Validate non-empty
    let meta = fs::metadata(dst_bin).map_err(|e| format!("{}: {}", dst_bin.display(), e))?;
    if meta.len() == 0 {
        return Err(format!("{} produced empty binary", dst_bin.display()));
    }

    Ok(())
}

fn find_tools() -> Option<(String, String, String)> {
    let prefix = env::var("RISCV_PREFIX").ok();
    let prefixes = match prefix {
        Some(p) => vec![p],
        None => vec![
            "riscv64-unknown-elf-".into(),
            "riscv64-linux-gnu-".into(),
            "riscv64-elf-".into(),
        ],
    };
    for p in prefixes {
        let as_cmd = format!("{p}as");
        let ld_cmd = format!("{p}ld");
        let objcopy_cmd = format!("{p}objcopy");
        if which::which(&as_cmd).is_ok()
            && which::which(&ld_cmd).is_ok()
            && which::which(&objcopy_cmd).is_ok()
        {
            return Some((as_cmd, ld_cmd, objcopy_cmd));
        }
    }
    None
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    let out = cmd.output().map_err(|e| format!("spawn {what}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{what} failed\ncmd: {:?}\nstdout:\n{}\nstderr:\n{}",
            cmd,
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    Ok(())
}
