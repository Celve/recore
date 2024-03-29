use core::mem::MaybeUninit;

use spin::Spin;

use crate::{
    config::{KERNEL_PAGE_NUM, KERNEL_START, PAGE_SIZE},
    mm::address::PhyPageNum,
};

use super::{normal::page::NormalPage, slab::page::SlabPage, user::UserPage};

pub static mut MEM_MAP: [MaybeUninit<Spin<Page>>; KERNEL_PAGE_NUM] =
    MaybeUninit::uninit_array::<KERNEL_PAGE_NUM>();

#[derive(Debug)]
pub enum Page {
    Slab(SlabPage),
    Normal(NormalPage),
    User(UserPage),
}

impl Page {
    pub fn from_pa(pa: usize) -> &'static Spin<Page> {
        unsafe { MEM_MAP[(pa - KERNEL_START) / PAGE_SIZE].assume_init_ref() }
    }

    pub fn from_ppn(ppn: PhyPageNum) -> &'static Spin<Page> {
        unsafe { MEM_MAP[ppn.0 - KERNEL_START / PAGE_SIZE].assume_init_ref() }
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

    pub fn as_normal(&self) -> &NormalPage {
        match self {
            Page::Normal(normal) => normal,
            _ => panic!("Page is not a normal page"),
        }
    }

    pub fn as_normal_mut(&mut self) -> &mut NormalPage {
        match self {
            Page::Normal(normal) => normal,
            _ => panic!("Page is not a normal page"),
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
