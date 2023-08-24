
use common::structures::exception_t;

use crate::cte::{cte_insert, cte_move, cte_swap, cap_removable, insert_new_cap};

use super::cap::{is_cap_revocable, same_object_as};

pub use crate::cap_rights::seL4_CapRights_t;
pub use crate::cte::cte_t;
pub use crate::mdb::mdb_node_t;
pub use crate::cap::cap_t;
pub use crate::cap::CapTag;

pub use super::cte::deriveCap_ret;
pub use super::cap::null::cap_null_cap_new;

pub use crate::cap_rights::{seL4_CapRightsBits, seL4_CapRights_get_capAllowGrant, seL4_CapRights_get_capAllowGrantReply,
    seL4_CapRights_get_capAllowRead, seL4_CapRights_get_capAllowWrite, rightsFromWord, wordFromRights};

pub use super::cap::untyped::{
    cap_untyped_cap_get_capBlockSize, cap_untyped_cap_get_capFreeIndex, cap_untyped_cap_get_capIsDevice,
    cap_untyped_cap_get_capPtr, cap_untyped_cap_new, cap_untyped_cap_ptr_set_capFreeIndex, cap_untyped_cap_set_capFreeIndex,
};

pub use super::cap::endpoint::{
    cap_endpoint_cap_get_capCanGrant, cap_endpoint_cap_get_capCanGrantReply, cap_endpoint_cap_get_capCanReceive,
    cap_endpoint_cap_get_capCanSend, cap_endpoint_cap_get_capEPBadge, cap_endpoint_cap_get_capEPPtr, cap_endpoint_cap_new,
    cap_endpoint_cap_set_capCanGrant, cap_endpoint_cap_set_capCanGrantReply, cap_endpoint_cap_set_capCanReceive,
    cap_endpoint_cap_set_capCanSend, cap_endpoint_cap_set_capEPBadge
};

pub use super::cap::zombie::{
    cap_zombie_cap_get_capZombieID, cap_zombie_cap_get_capZombieType, cap_zombie_cap_new, cap_zombie_cap_set_capZombieID,
    cap_zombie_cap_get_capZombieBits, cap_zombie_cap_get_capZombieNumber, cap_zombie_cap_get_capZombiePtr, Zombie_new,
    cap_zombie_cap_set_capZombieNumber, ZombieType_ZombieTCB
};

pub use super::cap::page_table::{
    cap_page_table_cap_get_capPTBasePtr, cap_page_table_cap_get_capPTIsMapped, cap_page_table_cap_get_capPTMappedASID,
    cap_page_table_cap_get_capPTMappedAddress, cap_page_table_cap_new, cap_page_table_cap_ptr_set_capPTIsMapped,
    cap_page_table_cap_set_capPTIsMapped, cap_page_table_cap_set_capPTMappedASID, cap_page_table_cap_set_capPTMappedAddress,
};

pub use super::cap::frame::{
    cap_frame_cap_get_capFBasePtr, cap_frame_cap_get_capFIsDevice, cap_frame_cap_get_capFMappedASID, cap_frame_cap_get_capFMappedAddress,
    cap_frame_cap_get_capFSize, cap_frame_cap_get_capFVMRights, cap_frame_cap_new, cap_frame_cap_set_capFMappedASID,
    cap_frame_cap_set_capFMappedAddress, cap_frame_cap_set_capFVMRights,
};

pub use super::cap::asid_control::cap_asid_control_cap_new;

pub use super::cap::asid_pool::{
    cap_asid_pool_cap_get_capASIDBase, cap_asid_pool_cap_get_capASIDPool, cap_asid_pool_cap_new
};

pub use super::cap::domain::cap_domain_cap_new;

pub use super::cap::reply::{
    cap_reply_cap_get_capReplyCanGrant, cap_reply_cap_get_capReplyMaster, cap_reply_cap_get_capTCBPtr, cap_reply_cap_new,
    cap_reply_cap_set_capReplyCanGrant,
};

pub use super::cap::thread::{
    cap_thread_cap_get_capTCBPtr, cap_thread_cap_new
};

pub use super::cap::notification::{
    cap_notification_cap_get_capNtfnBadge, cap_notification_cap_get_capNtfnCanReceive, cap_notification_cap_get_capNtfnCanSend,
    cap_notification_cap_get_capNtfnPtr, cap_notification_cap_new, cap_notification_cap_set_capNtfnBadge,
    cap_notification_cap_set_capNtfnCanReceive, cap_notification_cap_set_capNtfnCanSend, cap_notification_cap_set_capNtfnPtr,
};

pub use super::cap::cnode::{
    cap_cnode_cap_get_capCNodeGuard, cap_cnode_cap_get_capCNodeGuardSize, cap_cnode_cap_get_capCNodePtr,
    cap_cnode_cap_get_capCNodeRadix, cap_cnode_cap_new, cap_cnode_cap_set_capCNodeGuard, cap_cnode_cap_set_capCNodeGuardSize
};

pub use super::cap::irq_control::cap_irq_control_cap_new;

pub use super::cap::irq_handler::{
    cap_irq_handler_cap_get_capIRQ, cap_irq_handler_cap_new
};

//cap_tag_t
pub const cap_null_cap: usize = CapTag::CapNullCap as usize;
pub const cap_untyped_cap: usize = CapTag::CapUntypedCap as usize;
pub const cap_endpoint_cap: usize = CapTag::CapEndpointCap as usize;
pub const cap_notification_cap: usize = CapTag::CapNotificationCap as usize;
pub const cap_reply_cap: usize = CapTag::CapReplyCap as usize;
pub const cap_cnode_cap: usize = CapTag::CapCNodeCap as usize;
pub const cap_thread_cap: usize = CapTag::CapThreadCap as usize;
pub const cap_irq_control_cap: usize = CapTag::CapIrqControlCap as usize;
pub const cap_irq_handler_cap: usize = CapTag::CapIrqHandlerCap as usize;
pub const cap_zombie_cap: usize = CapTag::CapZombieCap as usize;
pub const cap_domain_cap: usize = CapTag::CapDomainCap as usize;
pub const cap_frame_cap: usize = CapTag::CapFrameCap as usize;
pub const cap_page_table_cap: usize = CapTag::CapPageTableCap as usize;
pub const cap_asid_control_cap: usize = CapTag::CapASIDControlCap as usize;
pub const cap_asid_pool_cap: usize = CapTag::CapASIDPoolCap as usize;


#[inline]
pub fn mdb_node_new(mdbNext: usize, mdbRevocable: usize, mdbFirstBadged: usize, mdbPrev: usize) -> mdb_node_t {
    mdb_node_t::new(mdbNext, mdbRevocable, mdbFirstBadged, mdbPrev)
}

#[inline]
pub fn mdb_node_get_mdbNext(mdb_node: &mdb_node_t) -> usize {
    mdb_node.get_next()
}

#[inline]
pub fn mdb_node_ptr_set_mdbNext(mdb_node: &mut mdb_node_t, v64: usize) {
    mdb_node.set_next(v64)
}

#[inline]
pub fn mdb_node_get_mdbRevocable(mdb_node: &mdb_node_t) -> usize {
    mdb_node.get_revocable()
}

#[inline]
pub fn mdb_node_get_mdbFirstBadged(mdb_node: &mdb_node_t) -> usize {
    mdb_node.get_first_badged()
}

#[inline]
pub fn mdb_node_set_mdbRevocable(mdb_node: &mut mdb_node_t, v64: usize) {
    mdb_node.set_revocable(v64)
}

#[inline]
pub fn mdb_node_set_mdbFirstBadged(mdb_node: &mut mdb_node_t, v64: usize) {
    mdb_node.set_first_badged(v64)
}

#[inline]
pub fn mdb_node_get_mdbPrev(mdb_node: &mdb_node_t) -> usize {
    mdb_node.get_prev()
}

#[inline]
pub fn mdb_node_set_mdbPrev(mdb_node: &mut mdb_node_t, v64: usize) {
    mdb_node.set_prev(v64)
}

#[inline]
pub fn mdb_node_ptr_set_mdbPrev(mdb_node: &mut mdb_node_t, v64: usize) {
    mdb_node.set_prev(v64)
}

#[inline]
pub fn cap_get_capType(cap: &cap_t) -> usize {
    cap.get_cap_type() as usize
}

pub fn cap_get_capPtr(cap: &cap_t) -> usize {
    cap.get_cap_ptr()
}

#[inline]
pub fn isArchCap(cap: &cap_t) -> bool {
    cap.isArchCap()
}


#[no_mangle]
pub fn ensureNoChildren(slot: *mut cte_t) -> exception_t {
    unsafe {
        (& *slot).ensure_no_children()
    }
}

#[no_mangle]
pub fn isMDBParentOf(cte1: *mut cte_t, cte2: *mut cte_t) -> bool {
    unsafe {
        (& *cte1).is_mdb_parent_of(& *cte2)
    }
}

#[no_mangle]
pub fn isFinalCapability(cte: *mut cte_t) -> bool {
    unsafe {
        (& *cte).is_final_cap()
    }
}


#[no_mangle]
pub fn slotCapLongRunningDelete(slot: *mut cte_t) -> bool {
    unsafe {
        (& *slot).is_long_running_delete()
    }
}

pub fn isCapRevocable(_derivedCap: &cap_t, _srcCap: &cap_t) -> bool {
    is_cap_revocable(_derivedCap, _srcCap)
}


#[no_mangle]
pub fn cteInsert(newCap: &cap_t, srcSlot: *mut cte_t, destSlot: *mut cte_t) {
    unsafe {
        cte_insert(newCap, &mut *srcSlot, &mut *destSlot)
    }
}

#[no_mangle]
pub fn deriveCap(slot: *mut cte_t, cap: &cap_t) -> deriveCap_ret {
    unsafe {
        (&mut *slot).derive_cap(cap)
    }
}

#[no_mangle]
pub fn cteMove(_newCap: &cap_t, srcSlot: *mut cte_t, destSlot: *mut cte_t) {
    unsafe {
        cte_move(_newCap, &mut *srcSlot, &mut *destSlot)
    }
}

#[no_mangle]
pub fn cteSwap(cap1: &cap_t, slot1: *mut cte_t, cap2: &cap_t, slot2: *mut cte_t) {
    unsafe {
        cte_swap(cap1, &mut *slot1, cap2, &mut *slot2)
    }
}


#[inline]
#[no_mangle]
pub fn capRemovable(cap: &cap_t, slot: *mut cte_t) -> bool {
    cap_removable(cap, slot)
}


#[no_mangle]
pub fn insertNewCap(parent: *mut cte_t, slot: *mut cte_t, cap: &cap_t) {
    unsafe {
        insert_new_cap(&mut *parent, &mut *slot, cap)
    }
}

pub fn sameObjectAs(cap_a: &cap_t, cap_b: &cap_t) -> bool {
    same_object_as(cap_a, cap_b)
}

#[inline]
pub fn cap_capType_equals(cap: &cap_t, cap_type_tag: usize) -> bool {
    cap.get_cap_type() as usize == cap_type_tag
}