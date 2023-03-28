use std::{
    env,
    fs::{self, File},
    io::{Result, Write},
    path::PathBuf,
};

fn main() {
    let ld = &PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("linker.ld");
    fs::write(ld, LINKER).unwrap();
    link();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", USER_APPS_BIN_DIR_PATH);
    println!("cargo:rerun-if-env-changed=LOG");
    println!("cargo:rustc-link-arg=-T{}", ld.display());
}

fn link() -> Result<()> {
    let mut names: Vec<String> = fs::read_dir(USER_APPS_SRC_DIR_PATH)
        .unwrap()
        .map(|dir_entry| {
            let name = dir_entry.unwrap().file_name().into_string().unwrap();
            name.get(0..name.find('.').unwrap()).unwrap().to_owned()
        })
        .filter(|name| !name.starts_with("disabled_"))
        .collect();
    names.sort();
    let n = names.len();

    let mut output = String::from("\t.align 3\n")
        + "\t.section .data\n"
        + "\t.global _num_apps\n"
        + "_num_apps:\n"
        + format!("\t.quad {n}\n").as_str();
    (0..n).for_each(|i| output += format!("\t.quad app_{i}_start\n").as_str());
    output += format!("\t.quad app_{}_end\n", n - 1).as_str();
    output += "\n";

    output += "\t.global _app_names\n";
    output += "_app_names:\n";
    names
        .iter()
        .for_each(|name| output += format!("\t.string \"{name}\"\n").as_str());
    output += "\n";

    names.iter().enumerate().for_each(|(i, name)| {
        output += "\t.section .data\n";
        output += format!("\t.global app_{i}_start\n").as_str();
        output += format!("\t.global app_{i}_end\n").as_str();
        output += "\t.align 3\n";
        output += format!("app_{i}_start:\n").as_str();
        output += format!("\t.incbin \"{}{}\"\n", USER_APPS_BIN_DIR_PATH, name).as_str();
        output += format!("app_{i}_end:\n").as_str();
        output += "\n";
    });

    let mut f = File::create("src/app.s").unwrap();
    writeln!(f, "{output}").unwrap();

    Ok(())
}

const USER_APPS_BIN_DIR_PATH: &str = "../user/target/riscv64gc-unknown-none-elf/release/";
const USER_APPS_SRC_DIR_PATH: &str = "../user/src/bin/";

const LINKER: &[u8] = b"
OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80000000;

SECTIONS
{
    . = BASE_ADDRESS;
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        etrampoline = .;
        . = ALIGN(4K);
        *(.text .text.*)
    }

    . = ALIGN(4K);
    etext = .;

    srodata = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    . = ALIGN(4K);
    erodata = .;

    sdata = .;
    .data : {
        *(.data.heap)
        *(.data .data.*)
        *(.sdata .sdata.*)
    }

    . = ALIGN(4K);
    edata = .;

    sbss_with_stack = .;
    .bss : {
        *(.bss.stack)
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }

    . = ALIGN(4K);
    ebss = .;
    
    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
";
