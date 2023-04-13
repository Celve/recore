use alloc::vec::Vec;
use lazy_static::lazy_static;

pub fn get_num_apps() -> usize {
    extern "C" {
        fn _num_apps();
    }
    unsafe { (_num_apps as *const usize).read_volatile() }
}

/// The function churns out app name vec at the beginning.
pub fn get_app_names() -> Vec<&'static str> {
    let num_apps = get_num_apps();

    extern "C" {
        fn _app_names();
    }
    let mut start = _app_names as usize as *const u8;
    let mut app_names = Vec::new();
    unsafe {
        for _ in 0..num_apps {
            let mut end = start;
            let mut c = end.read_volatile();
            while c != '\0' as u8 {
                end = end.add(1);
                c = end.read_volatile();
            }
            let bytes = core::slice::from_raw_parts(start, end as usize - start as usize);
            let str = core::str::from_utf8(bytes).unwrap();
            app_names.push(str);
            start = end.add(1);
        }
    }
    app_names
}

pub fn get_app_data(name: &str) -> Option<&'static [u8]> {
    extern "C" {
        fn _num_apps();
    }
    let id = APP_NAMES.iter().position(|&x| x == name)?;
    unsafe {
        let base_addr = _num_apps as *const usize;
        let num_apps = base_addr.read_volatile();
        let app_addrs = core::slice::from_raw_parts(base_addr.add(1), num_apps + 1);
        Some(core::slice::from_raw_parts(
            app_addrs[id] as *const u8,
            app_addrs[id + 1] - app_addrs[id],
        ))
    }
}

lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = get_app_names();
}
