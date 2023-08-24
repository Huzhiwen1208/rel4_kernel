use core::intrinsics::{likely, unlikely};

use crate::{
    config::{seL4_FailedLookup, seL4_RangeError, tcbCTable},
    println,
    structures::{
        lookupCapAndSlot_ret_t, lookupCap_ret_t, lookupSlot_raw_ret_t,
        lookupSlot_ret_t, resolveAddressBits_ret_t, tcb_t,
    },
};

use common::{structures::{exception_t, lookup_fault_invalid_root_new, lookup_fault_guard_mismatch_new, lookup_fault_depth_mismatch_new}, sel4_config::{wordRadix, wordBits}, MASK};
use cspace::interface::*;

use super::{
    boot::{current_lookup_fault, current_syscall_error},
    thread::getCSpace,
};

#[no_mangle]
pub extern "C" fn lookupSlot(thread: *const tcb_t, capptr: usize) -> lookupSlot_raw_ret_t {
    unsafe {
        let threadRoot = &(*getCSpace(thread as usize, tcbCTable)).cap;
        let res_ret = rust_resolveAddressBits(threadRoot, capptr, wordBits);
        let ret = lookupSlot_raw_ret_t {
            status: res_ret.status,
            slot: res_ret.slot,
        };
        return ret;
    }
}

#[no_mangle]
pub extern "C" fn lookupCapAndSlot(thread: *const tcb_t, cPtr: usize) -> lookupCapAndSlot_ret_t {
    let lu_ret = lookupSlot(thread, cPtr);
    if lu_ret.status != exception_t::EXCEPTION_NONE {
        let ret = lookupCapAndSlot_ret_t {
            status: lu_ret.status,
            slot: 0 as *mut cte_t,
            cap: cap_null_cap_new(),
        };
        return ret;
    }
    unsafe {
        let ret = lookupCapAndSlot_ret_t {
            status: exception_t::EXCEPTION_NONE,
            slot: lu_ret.slot,
            cap: (*lu_ret.slot).cap.clone(),
        };
        ret
    }
}

#[no_mangle]
pub extern "C" fn rust_resolveAddressBits(
    _nodeCap: &cap_t,
    capptr: usize,
    _n_bits: usize,
) -> resolveAddressBits_ret_t {
    unsafe {
        let mut ret = resolveAddressBits_ret_t::default();
        let mut n_bits = _n_bits;
        ret.bitsRemaining = n_bits;
        let mut radixBits: usize;
        let mut guardBits: usize;
        let mut guard: usize;
        let mut levelBits: usize;
        let mut capGuard: usize;
        let mut offset: usize;
        let mut slot: *mut cte_t;
        let mut nodeCap = _nodeCap.clone();
        if unlikely(cap_get_capType(&nodeCap) != cap_cnode_cap) {
            current_lookup_fault = lookup_fault_invalid_root_new();
            ret.status = exception_t::EXCEPTION_LOOKUP_FAULT;
            return ret;
        }

        while true {
            radixBits = cap_cnode_cap_get_capCNodeRadix(&nodeCap);
            guardBits = cap_cnode_cap_get_capCNodeGuardSize(&nodeCap);
            levelBits = radixBits + guardBits;

            assert!(levelBits != 0);
            capGuard = cap_cnode_cap_get_capCNodeGuard(&nodeCap);
            guard = (capptr >> ((n_bits - guardBits) & MASK!(wordRadix))) & MASK!(guardBits);
            if unlikely(guardBits > n_bits || guard != capGuard) {
                current_lookup_fault = lookup_fault_guard_mismatch_new(capGuard, n_bits, guardBits);
                ret.status = exception_t::EXCEPTION_LOOKUP_FAULT;
                return ret;
            }

            if unlikely(levelBits > n_bits) {
                current_lookup_fault = lookup_fault_depth_mismatch_new(levelBits, n_bits);
                ret.status = exception_t::EXCEPTION_LOOKUP_FAULT;
                return ret;
            }

            offset = (capptr >> (n_bits - levelBits)) & MASK!(radixBits);
            slot = ((cap_cnode_cap_get_capCNodePtr(&nodeCap)) as *mut cte_t).add(offset);

            if likely(n_bits == levelBits) {
                ret.slot = slot;
                ret.bitsRemaining = 0;
                return ret;
            }
            n_bits -= levelBits;
            nodeCap = (*slot).cap.clone();
            if unlikely(cap_get_capType(&nodeCap) != cap_cnode_cap) {
                ret.slot = slot;
                ret.bitsRemaining = n_bits;
                return ret;
            }
        }
        panic!("UNREACHABLE");
    }
}

pub fn lookupSlotForCNodeOp(
    isSource: bool,
    root: &cap_t,
    capptr: usize,
    depth: usize,
) -> lookupSlot_ret_t {
    let mut ret: lookupSlot_ret_t = lookupSlot_ret_t::default();
    if unlikely(cap_get_capType(&root) != cap_cnode_cap) {
        unsafe {
            current_syscall_error._type = seL4_FailedLookup;
            current_syscall_error.failedLookupWasSource = isSource as usize;
            current_lookup_fault = lookup_fault_invalid_root_new();
        }
        println!("in here1");
        ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
        return ret;
    }
    if unlikely(depth < 1 || depth > wordBits) {
        unsafe {
            current_syscall_error._type = seL4_RangeError;
            current_syscall_error.rangeErrorMin = 1;
            current_syscall_error.rangeErrorMax = wordBits;
        }
        println!("in here2");
        ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
        return ret;
    }

    let res_ret = rust_resolveAddressBits(&root, capptr, depth);

    if unlikely(ret.status != exception_t::EXCEPTION_NONE) {
        unsafe {
            current_syscall_error._type = seL4_FailedLookup;
            current_syscall_error.failedLookupWasSource = isSource as usize;
        }
        ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
        println!("in here3");
        return ret;
    }

    if unlikely(res_ret.bitsRemaining != 0) {
        unsafe {
            current_syscall_error._type = seL4_FailedLookup;
            current_syscall_error.failedLookupWasSource = isSource as usize;
            current_lookup_fault = lookup_fault_depth_mismatch_new(0, res_ret.bitsRemaining);
        }
        ret.status = exception_t::EXCEPTION_SYSCALL_ERROR;
        println!("in here4");
        return ret;
    }
    ret.slot = res_ret.slot;
    ret.status = exception_t::EXCEPTION_NONE;
    return ret;
}

#[no_mangle]
pub extern "C" fn lookupCap(thread: *const tcb_t, cPtr: usize) -> lookupCap_ret_t {
    let lu_ret = lookupSlot(thread, cPtr);
    if lu_ret.status != exception_t::EXCEPTION_NONE {
        return lookupCap_ret_t {
            status: lu_ret.status,
            cap: cap_null_cap_new(),
        };
    }
    unsafe {
        lookupCap_ret_t {
            status: exception_t::EXCEPTION_NONE,
            cap: (*lu_ret.slot).cap.clone(),
        }
    }
}

#[no_mangle]
pub extern "C" fn rust_lookupTargetSlot(
    root: &cap_t,
    capptr: usize,
    depth: usize,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(false, root, capptr, depth)
}

#[no_mangle]
pub extern "C" fn rust_lookupSourceSlot(
    root: &cap_t,
    capptr: usize,
    depth: usize,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(true, root, capptr, depth)
}

#[no_mangle]
pub extern "C" fn rust_lookupPivotSlot(
    root: &cap_t,
    capptr: usize,
    depth: usize,
) -> lookupSlot_ret_t {
    lookupSlotForCNodeOp(true, root, capptr, depth)
}
