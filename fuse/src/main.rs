use std::{
    fs::{read_dir, File},
    io::Read,
};

use clap::{App, Arg};
use fosix::fs::OpenFlags;
use fuse::FUSE;

use crate::cache::CACHE_MANAGER;

mod bitmap;
mod cache;
mod config;
mod dir;
mod disk;
mod file;
mod fuse;
mod inode;
mod superblock;

fn main() {
    let matches = App::new("Fuse packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Executable source dir"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();

    FUSE.alloc_root();
    let root = FUSE.root();

    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    let apps: Vec<String> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();

    for app in apps {
        // load app data from host file system
        let mut host_file = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut all_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut all_data).unwrap();

        // create a file in easy-fs
        root.touch(app.as_str()).unwrap();
        let mut inode = root.open(app.as_str(), OpenFlags::RDWR).unwrap();

        // write data to easy-fs
        let size = inode.write(&mut all_data);
        assert_eq!(size, all_data.len());
    }

    drop(root);
    CACHE_MANAGER.lock().clear();
    assert_eq!(CACHE_MANAGER.lock().len(), 0);
}

#[cfg(test)]
fn test() {
    let root = FUSE.lock().root();
    for i in 0..129 {
        root.mkdir(format!("{i}").as_str());
    }
    let res = root.ls();
    res.iter()
        .enumerate()
        .for_each(|(i, name)| assert_eq!(name, format!("{i}").as_str()));

    let one = root.cd("1").unwrap();
    let mut s = String::new();
    for i in 0..(8 * 1024 * 1024) {
        s.push('a');
    }
    one.touch("a").unwrap();
    let mut f = one.open("a").unwrap();
    println!("{}", f.write(s.as_bytes()));
    let mut v = vec![0u8; 8 * 1024 * 1024];
    f.read_at(v.as_mut(), 0);
    assert!(v.iter().any(|x| *x == 'a' as u8));
}
