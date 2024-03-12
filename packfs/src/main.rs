mod disk;

use std::{
    fs::{read_dir, File, OpenOptions},
    io::Read,
    sync::Arc,
};

use clap::{App, Arg};
use fosix::fs::OpenFlags;
use simplefs::{cache::CacheManager, fs::FileSys, superblock::SuperBlock};

use crate::disk::FileDev;

const NUM_INODE: usize = 8192;
const NUM_DNODE: usize = 65536;
const FILE_LEN: usize = 1 + NUM_INODE / 4096 + NUM_DNODE / 4096 + NUM_INODE / 4 + NUM_DNODE;

fn main() {
    let matches = App::new("File system packer")
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

    let disk_manager = Arc::new(FileDev::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("{}", "fs.img"))
            .unwrap();
        f.set_len(FILE_LEN as u64 * 512).unwrap();
        f
    }));

    let fs = Arc::new(FileSys::new(
        SuperBlock::new(NUM_INODE, NUM_DNODE),
        Arc::new(CacheManager::new(disk_manager)),
    ));

    fs.alloc_root();
    let root = fs.root();

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
        root.lock().touch(app.as_str()).unwrap();
        let inode = root.lock().open(app.as_str(), OpenFlags::RDWR).unwrap();

        // write data to easy-fs
        let size = inode.lock().write(&mut all_data);
        assert_eq!(size, all_data.len());
    }

    drop(root);
    let cache_manager = fs.cache_manager();
    cache_manager.clear();
    assert_eq!(cache_manager.len(), 0);
}

fn test() {
    let disk_manager = Arc::new(FileDev::new({
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(format!("{}", "fs.img"))
            .unwrap();
        f.set_len(33802 * 512).unwrap();
        f
    }));

    let fs = Arc::new(FileSys::new(
        SuperBlock::new(4096, 32768),
        Arc::new(CacheManager::new(disk_manager)),
    ));

    fs.alloc_root();
    let root = fs.root();

    for i in 0..129 {
        root.lock().mkdir(format!("{i}").as_str());
    }
    let res = root.lock().ls();
    res.iter().enumerate().for_each(|(i, name)| {
        if name != "." && name != ".." {
            assert_eq!(name, format!("{}", i - 2).as_str())
        }
    });

    let one = root.lock().cd("1").unwrap();
    let mut s = String::new();
    for _ in 0..(8 * 1024 * 1024) {
        s.push('a');
    }
    one.lock().touch("a").unwrap();
    let f = one.lock().open("a", OpenFlags::RDWR).unwrap();
    let written = f.lock().write(s.as_bytes());
    let mut v = vec![0u8; 8 * 1024 * 1024];
    assert_eq!(f.lock().read_at(v.as_mut(), 0), written);
    assert!(v.iter().any(|x| *x == 'a' as u8));

    assert_eq!(f.lock().trunc(), written);
    assert_eq!(f.lock().stat().size(), 0);

    let written = f.lock().write(s.as_bytes());
    let mut v = vec![0u8; 8 * 1024 * 1024];
    assert_eq!(f.lock().read_at(v.as_mut(), 0), written);
    assert!(v.iter().any(|x| *x == 'a' as u8));
}
