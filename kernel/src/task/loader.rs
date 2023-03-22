pub fn get_num_apps() -> usize {
    extern "C" {
        fn _num_apps();
    }
    unsafe { (_num_apps as *const usize).read_volatile() }
}

pub fn get_app_data(id: usize) -> &'static [u8] {
    extern "C" {
        fn _num_apps();
    }
    unsafe {
        let base_addr = _num_apps as *const usize;
        let num_apps = base_addr.read_volatile();
        let app_addrs = core::slice::from_raw_parts(base_addr.add(1), num_apps + 1);
        core::slice::from_raw_parts(
            app_addrs[id] as *const u8,
            app_addrs[id + 1] - app_addrs[id],
        )
    }
}
