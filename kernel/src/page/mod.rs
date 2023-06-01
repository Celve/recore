use core::mem::MaybeUninit;

use crate::config::{KERNEL_PAGE_NUM, KERNEL_START, PAGE_SIZE};

use self::{empty::EmptyPage, pt::PtPage, slab::SlabPage, user::UserPage};

pub mod empty;
pub mod pt;
pub mod slab;
pub mod user;

pub static mut MEM_MAP: [MaybeUninit<Page>; KERNEL_PAGE_NUM] =
    MaybeUninit::uninit_array::<KERNEL_PAGE_NUM>();

#[derive(Debug)]
pub enum Page {
    Empty(EmptyPage),
    Slab(SlabPage),
    Pt(PtPage),
    User(UserPage),
}

impl Page {
    /// Init the `MEM_MAP`. Otherwise, it's uninit.
    pub fn init_mem_map() {
        for i in 0..KERNEL_PAGE_NUM {
            let pa = KERNEL_START + i * PAGE_SIZE;
            unsafe {
                MEM_MAP[i] = MaybeUninit::new(Page::Empty(EmptyPage::new(pa)));
            }
        }
    }
}

impl Page {
    pub fn from_addr(pa: usize) -> &'static Page {
        unsafe { &MEM_MAP[(pa - KERNEL_START) / PAGE_SIZE].assume_init_ref() }
    }

    pub fn from_addr_mut(pa: usize) -> &'static mut Page {
        unsafe { MEM_MAP[(pa - KERNEL_START) / PAGE_SIZE].assume_init_mut() }
    }
}

impl Page {
    pub fn as_slab(&self) -> &SlabPage {
        match self {
            Page::Slab(slab) => slab,
            _ => panic!("Page is not a slab page"),
        }
    }

    pub fn as_slab_mut(&mut self) -> &mut SlabPage {
        match self {
            Page::Slab(slab) => slab,
            _ => panic!("Page is not a slab page"),
        }
    }

    pub fn as_pt(&self) -> &PtPage {
        match self {
            Page::Pt(pt) => pt,
            _ => panic!("Page is not a page table page"),
        }
    }

    pub fn as_pt_mut(&mut self) -> &mut PtPage {
        match self {
            Page::Pt(pt) => pt,
            _ => panic!("Page is not a page table page"),
        }
    }

    pub fn as_user(&self) -> &UserPage {
        match self {
            Page::User(user) => user,
            _ => panic!("Page is not a user page"),
        }
    }

    pub fn as_user_mut(&mut self) -> &mut UserPage {
        match self {
            Page::User(user) => user,
            _ => panic!("Page is not a user page"),
        }
    }
}
