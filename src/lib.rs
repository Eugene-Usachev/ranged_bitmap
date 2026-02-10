#![no_std]

extern crate core;

#[cfg(test)]
extern crate alloc;

pub(crate) mod base;
pub mod fixed;

pub(crate) const fn maybe_assert(res: bool, msg: &'static str) {
    if cfg!(any(debug_assertions, feature = "more_checks")) {
        if !res {
            panic!("{}", msg);
        }
    }
}
