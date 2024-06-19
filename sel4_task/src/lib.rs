//! This crate provides task management for seL4, including the TCB, scheduler, and thread relevant structures.

#![feature(core_intrinsics)]
#![no_std]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]

mod thread_state;
mod tcb_queue;
pub mod tcb;
mod structures;
mod scheduler;
mod deps;

pub use tcb::*;
pub use scheduler::*;
pub use thread_state::*;
pub use tcb_queue::*;
pub use structures::*;