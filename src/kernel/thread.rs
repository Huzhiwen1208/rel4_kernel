use crate::{
    config::{
        seL4_TCBBits, wordBits, wordRadix, SchedulerAction_ChooseNewThread,
        SchedulerAction_ResumeCurrentThread, ThreadStateIdleThreadState, ThreadStateInactive,
        ThreadStateRestart, ThreadStateRunning, CONFIG_KERNEL_STACK_BITS, CONFIG_MAX_NUM_NODES,
        CONFIG_NUM_DOMAINS, CONFIG_NUM_PRIORITIES, L2_BITMAP_SIZE, NUM_READY_QUEUES, SSTATUS_SPIE,
        SSTATUS_SPP,
    },
    object::structure_gen::{
        thread_state_get_tcbQueued, thread_state_get_tsType, thread_state_set_tcbQueued,
        thread_state_set_tsType,
    },
    println,
    sbi::shutdown,
    structures::{arch_tcb_t, cte_t, tcb_queue_t, tcb_t},
    BIT, MASK,
};

use core::arch::asm;

use super::vspace::setVMRoot;

pub static mut ksDomainTime: usize = 0;

pub static mut ksCurDomain: usize = 0;

pub static mut ksDomScheduleIdx: usize = 0;

pub static mut ksCurThread: usize = 0;

pub static mut ksIdleThread: usize = 0;

pub static mut ksSchedulerAction: usize = 1;


#[no_mangle]
pub static mut kernel_stack_alloc: [[u8; BIT!(CONFIG_KERNEL_STACK_BITS)]; CONFIG_MAX_NUM_NODES] =
    [[0; BIT!(CONFIG_KERNEL_STACK_BITS)]; CONFIG_MAX_NUM_NODES];

static mut ksReadyQueues: [tcb_queue_t; NUM_READY_QUEUES] =
    [tcb_queue_t { head: 0, tail: 0 }; NUM_READY_QUEUES];

static mut ksReadyQueuesL2Bitmap: [[usize; L2_BITMAP_SIZE]; CONFIG_NUM_DOMAINS] =
    [[0; L2_BITMAP_SIZE]; CONFIG_NUM_DOMAINS];

static mut ksReadyQueuesL1Bitmap: [usize; CONFIG_NUM_DOMAINS] = [0; CONFIG_NUM_DOMAINS];

#[link_section = "._idle_thread"]
pub static mut ksIdleThreadTCB: [[u8; BIT!(seL4_TCBBits)]; CONFIG_MAX_NUM_NODES] =
    [[0; BIT!(seL4_TCBBits)]; CONFIG_MAX_NUM_NODES];

type prio_t = usize;
pub const ra: usize = 0;
pub const sp: usize = 1;
const gp: usize = 2;
const tp: usize = 3;
const t0: usize = 4;
const t1: usize = 5;
const t2: usize = 6;
const s0: usize = 7;
const s1: usize = 8;
const a0: usize = 9;
pub const capRegister: usize = 9;
pub const badgeRegister: usize = 9;
const a1: usize = 10;
const a2: usize = 11;
const a3: usize = 12;
const a4: usize = 13;
const a5: usize = 14;
const a6: usize = 15;
const a7: usize = 16;
const s2: usize = 17;
const s3: usize = 18;
const s4: usize = 19;
const s5: usize = 20;
const s6: usize = 21;
const s7: usize = 22;
const s8: usize = 23;
const s9: usize = 24;
const s10: usize = 25;
const s11: usize = 26;
const t3: usize = 27;
const t4: usize = 28;
const t5: usize = 29;
const t6: usize = 30;
pub const SCAUSE: usize = 31;
pub const SSTATUS: usize = 32;
pub const FaultIP: usize = 33;
pub const NextIP: usize = 34;
pub const n_contextRegisters: usize = 35;

#[inline]
pub fn isStopped(thread: *const tcb_t) -> bool {
    if thread as usize == 0 || thread as usize == 1 {
        return true;
    }
    unsafe {
        match thread_state_get_tsType(&(*thread).tcbState) {
            ThreadStateInactive => true,
            ThreadStateBlockedOnNotification => true,
            ThreadStateBlockedOnReceive => true,
            ThreadStateBlockedOnReply => true,
            ThreadStateBlockedOnSend => true,
            _ => false,
        }
    }
}

#[inline]
pub fn isRunnable(thread: *const tcb_t) -> bool {
    if thread as usize == 0 || thread as usize == 1 {
        return false;
    }
    unsafe {
        match thread_state_get_tsType(&(*thread).tcbState) {
            ThreadStateRunning => true,
            ThreadStateRestart => true,
            _ => false,
        }
    }
}

#[inline]
pub fn setRegister(thread: *mut tcb_t, reg: usize, w: usize) {
    unsafe {
        (*thread).tcbArch.registers[reg] = w;
    }
}

#[inline]
pub fn getRegister(thread: *const tcb_t, reg: usize) -> usize {
    unsafe { (*thread).tcbArch.registers[reg] }
}

#[inline]
pub fn ready_queues_index(dom: usize, prio: usize) -> usize {
    dom * CONFIG_NUM_PRIORITIES + prio
}

#[inline]
pub fn prio_to_l1index(prio: usize) -> usize {
    prio >> wordRadix
}

#[inline]
pub fn l1index_to_prio(l1index: usize) -> usize {
    l1index << wordRadix
}

#[inline]
pub fn invert_l1index(l1index: usize) -> usize {
    let inverted = L2_BITMAP_SIZE - 1 - l1index;
    inverted
}

#[inline]
pub fn getHighestPrio(dom: usize) -> prio_t {
    unsafe {
        let l1index = wordBits - 1 - ksReadyQueuesL1Bitmap[dom].leading_zeros() as usize;
        let l1index_inverted = invert_l1index(l1index);
        let l2index =
            wordBits - 1 - ksReadyQueuesL2Bitmap[dom][l1index_inverted].leading_zeros() as usize;
        l1index_to_prio(l1index) | l2index
    }
}

#[inline]
pub fn isHighestPrio(dom: usize, prio: prio_t) -> bool {
    unsafe { ksReadyQueuesL1Bitmap[dom] == 0 || prio >= getHighestPrio(dom) }
}

#[inline]
pub fn addToBitmap(dom: usize, prio: usize) {
    unsafe {
        let l1index = prio_to_l1index(prio);
        let l1index_inverted = invert_l1index(l1index);
        ksReadyQueuesL1Bitmap[dom] |= BIT!(l1index);
        ksReadyQueuesL2Bitmap[dom][l1index_inverted] |= BIT!(prio & MASK!(wordRadix));
    }
}

#[inline]
pub fn removeFromBitmap(dom: usize, prio: usize) {
    unsafe {
        let l1index = prio_to_l1index(prio);
        let l1index_inverted = invert_l1index(l1index);
        ksReadyQueuesL2Bitmap[dom][l1index_inverted] &= !BIT!(prio & MASK!(wordRadix));
        if ksReadyQueuesL2Bitmap[dom][l1index_inverted] == 0 {
            ksReadyQueuesL1Bitmap[dom] &= !(BIT!((l1index)));
        }
    }
}

pub fn idle_thread() {
    unsafe {
        while true {
            asm!("wfi");
        }
    }
}

pub fn Arch_configureIdleThread(tcb: *const tcb_t) {
    setRegister(tcb as *mut tcb_t, NextIP, idle_thread as usize);
    setRegister(tcb as *mut tcb_t, SSTATUS, SSTATUS_SPP | SSTATUS_SPIE);
    unsafe {
        setRegister(
            tcb as *mut tcb_t,
            sp,
            kernel_stack_alloc.as_ptr() as usize + BIT!(CONFIG_KERNEL_STACK_BITS),
        );
    }
}

pub fn Arch_switchToIdleThread() {
    unsafe {
        let tcb = ksIdleThread as *mut tcb_t;
        setVMRoot(tcb);
    }
}

pub fn setThreadState(tptr: *mut tcb_t, ts: usize) {
    unsafe {
        thread_state_set_tsType(&mut (*tptr).tcbState, ts);
        scheduleTCB(tptr);
    }
}

pub fn scheduleTCB(tptr: *const tcb_t) {
    unsafe {
        if tptr as usize == ksCurThread
            && ksSchedulerAction == SchedulerAction_ResumeCurrentThread
            && !isRunnable(tptr)
        {
            rescheduleRequired();
        }
    }
}

pub fn getReStartPC(thread: *const tcb_t) -> usize {
    getRegister(thread, FaultIP)
}

pub fn setRestartPC(thread: *mut tcb_t, v: usize) {
    setRegister(thread, NextIP, v);
}

pub fn setNextPC(thread: *mut tcb_t, v: usize) {
    setRegister(thread, NextIP, v);
}

pub fn configureIdleThread(tcb: *const tcb_t) {
    Arch_configureIdleThread(tcb);
    setThreadState(tcb as *mut tcb_t, ThreadStateIdleThreadState);
}

pub fn getCSpace(ptr: usize, i: usize) -> *mut cte_t {
    unsafe {
        let p = (ptr & !MASK!(seL4_TCBBits)) as *mut cte_t;
        p.add(i)
    }
}

pub fn rescheduleRequired() {
    unsafe {
        if ksSchedulerAction != SchedulerAction_ResumeCurrentThread
            && ksSchedulerAction != SchedulerAction_ChooseNewThread
        {
            tcbSchedEnqueue(ksSchedulerAction as *mut tcb_t);
        }
        ksSchedulerAction = SchedulerAction_ChooseNewThread;
    }
}

pub fn Arch_switchToThread(tcb: *const tcb_t) {
    setVMRoot(tcb as *mut tcb_t);
}

pub fn activateThread() {
    unsafe {
        assert!(ksCurThread != 0 && ksCurThread != 1);
        let thread = ksCurThread as *mut tcb_t;
        match thread_state_get_tsType(&(*thread).tcbState) {
            ThreadStateRunning => {
                Arch_switchToThread(thread);
            }
            ThreadStateRestart => {
                let pc = getReStartPC(thread as *const tcb_t);
                setNextPC(thread, pc);
                setThreadState(thread as *mut tcb_t, ThreadStateRunning);
                Arch_switchToThread(thread);
            }
            ThreadStateIdleThreadState => return,
            _ => panic!(
                "current thread is blocked , state id :{}",
                thread_state_get_tsType(&(*thread).tcbState)
            ),
        }
    }
}

#[inline]
pub fn updateReStartPC(tcb: *mut tcb_t) {
    setRegister(tcb, FaultIP, getRegister(tcb, NextIP));
}

pub fn suspend(target: *mut tcb_t) {
    //FIXME::implement cancelIPC;
    // cancelIPC(target);
    unsafe {
        if thread_state_get_tsType(&(*target).tcbState) == ThreadStateRunning {
            updateReStartPC(target);
        }
        setThreadState(target, ThreadStateInactive);
        tcbSchedDequeue(target);
    }
}

pub fn restart(target: *mut tcb_t) {
    if isStopped(target) {
        // cancelIPC(target);
        // FIXME::implemented setupReplyMaster
        // setupReplyMaster(target);
        setThreadState(target, ThreadStateRestart);
        tcbSchedEnqueue(target);
        possibleSwitchTo(target);
    }
}

pub fn doNBRecvFailedTransfer(thread: *mut tcb_t) {
    setRegister(thread, badgeRegister, 0);
}

// pub fn nextDomain() {
//     unsafe {
//         ksDomScheduleIdx += 1;
//         if ksDomScheduleIdx>=ksDomScheduleLength{
//             ksDomScheduleIdx=0;
//         }
//         //FIXME ksWorkUnits not used;
//         // ksWorkUnits
//     }
// }

pub fn scheduleChooseNewThread() {
    chooseThread();
}

pub fn switchToThread(thread: *const tcb_t) {
    Arch_switchToThread(thread);
    tcbSchedDequeue(thread as *mut tcb_t);
    unsafe {
        ksCurThread = thread as usize;
    }
}

pub fn chooseThread() {
    unsafe {
        let dom = 0;
        if ksReadyQueuesL1Bitmap[dom] != 0 {
            let prio = getHighestPrio(dom);
            // println!("prio:{}", prio);
            let _thread = ksReadyQueues[ready_queues_index(dom, prio)].head;
            assert!(_thread != 0);
            let thread = _thread as *const tcb_t;
            switchToThread(thread);
        } else {
            // println!("[kernel] all applications finished! turn to shutdown");
            println!("in idle thread ,waiting for interrupt");
            shutdown();
        }
    }
}

pub fn switchToIdleThread() {
    unsafe {
        Arch_switchToIdleThread();
        ksCurThread = ksIdleThread;
    }
}

pub fn setMCPriority(tptr: *mut tcb_t, mcp: usize) {
    unsafe {
        (*tptr).tcbMCP = mcp;
    }
}

pub fn setPriority(tptr: *const tcb_t, prio: usize) {
    unsafe {
        tcbSchedDequeue(tptr as *mut tcb_t);
        let mut_tptr = tptr as *mut tcb_t;
        (*mut_tptr).tcbPriority = prio;
        if isRunnable(tptr) {
            if tptr as usize == ksCurThread {
                rescheduleRequired();
            } else {
                possibleSwitchTo(tptr);
            }
        }
    }
}

pub fn possibleSwitchTo(target: *const tcb_t) {
    unsafe {
        if ksCurDomain != (*target).domain {
            tcbSchedEnqueue(target as *mut tcb_t);
        } else if ksSchedulerAction != SchedulerAction_ResumeCurrentThread {
            rescheduleRequired();
            tcbSchedEnqueue(target as *mut tcb_t);
        } else {
            ksSchedulerAction = target as usize;
        }
    }
}

pub fn tcbSchedEnqueue(_tcb: *mut tcb_t) {
    unsafe {
        let tcb = &mut (*_tcb);
        if thread_state_get_tcbQueued(&tcb.tcbState) == 0 {
            let dom = tcb.domain;
            let prio = tcb.tcbPriority;
            let idx = ready_queues_index(dom, prio);
            let mut queue = ksReadyQueues[idx];
            if queue.tail == 0 {
                queue.head = _tcb as *const tcb_t as usize;
                addToBitmap(dom, prio);
            } else {
                (*(queue.tail as *mut tcb_t)).tcbSchedNext = tcb as *const tcb_t as usize;
            }
            (*_tcb).tcbSchedPrev = queue.tail;
            (*_tcb).tcbSchedNext = 0;
            queue.tail = tcb as *const tcb_t as usize;
            ksReadyQueues[idx] = queue;

            thread_state_set_tcbQueued(&mut tcb.tcbState, 1);
        }
    }
}

#[inline]
pub fn tcbSchedDequeue(_tcb: *mut tcb_t) {
    unsafe {
        let tcb = &mut (*_tcb);
        if thread_state_get_tcbQueued(&tcb.tcbState) != 0 {
            let dom = tcb.domain;
            let prio = tcb.tcbPriority;
            let idx = ready_queues_index(dom, prio);
            let mut queue = ksReadyQueues[idx];
            if tcb.tcbSchedPrev != 0 {
                (*(tcb.tcbSchedPrev as *mut tcb_t)).tcbSchedNext = tcb.tcbSchedNext;
            } else {
                queue.head = tcb.tcbSchedNext;
                if tcb.tcbSchedNext == 0 {
                    removeFromBitmap(dom, prio);
                }
            }
            if tcb.tcbSchedNext != 0 {
                (*(tcb.tcbSchedNext as *mut tcb_t)).tcbSchedPrev = tcb.tcbSchedPrev;
            } else {
                queue.tail = tcb.tcbSchedPrev;
            }

            ksReadyQueues[idx] = queue;
            thread_state_set_tcbQueued(&mut tcb.tcbState, 0);
        }
    }
}

pub fn tcbSchedAppend(tcb: *mut tcb_t) {
    unsafe {
        if thread_state_get_tcbQueued(&(*tcb).tcbState) == 0 {
            let dom = (*tcb).domain;
            let prio = (*tcb).tcbPriority;
            let idx = ready_queues_index(dom, prio);
            let mut queue = ksReadyQueues[idx];
            // println!("tail:{:#x} head:{:#x}", queue.tail, queue.head);
            if queue.head == 0 {
                queue.head = tcb as usize;
                addToBitmap(dom, prio);
            } else {
                let next = queue.tail as *mut tcb_t;
                (*next).tcbSchedNext = tcb as usize;
            }
            // println!("tail:{:#x} head:{:#x}", queue.tail, queue.head);
            (*tcb).tcbSchedPrev = queue.tail;
            (*tcb).tcbSchedNext = 0;
            ksReadyQueues[idx] = queue;

            thread_state_set_tcbQueued(&mut (*tcb).tcbState, 1);
        }
    }
}

pub fn tcbEPAppend(tcb: *mut tcb_t, mut queue: tcb_queue_t) -> tcb_queue_t {
    unsafe {
        if queue.head == 0 {
            queue.head = tcb as usize;
        } else {
            (*(queue.tail as *mut tcb_t)).tcbEPNext = tcb as usize;
        }
        (*tcb).tcbEPPrev = queue.tail;
        (*tcb).tcbEPNext = 0;
        queue.tail = tcb as usize;
        queue
    }
}

pub fn tcbEPDequeue(tcb: *mut tcb_t, mut queue: tcb_queue_t) -> tcb_queue_t {
    unsafe {
        if (*tcb).tcbEPPrev != 0 {
            (*((*tcb).tcbEPPrev as *mut tcb_t)).tcbEPNext = (*tcb).tcbEPNext;
        } else {
            queue.head = (*tcb).tcbEPNext as usize;
        }
        if (*tcb).tcbEPNext != 0 {
            (*((*tcb).tcbEPNext as *mut tcb_t)).tcbEPPrev = (*tcb).tcbEPPrev;
        } else {
            queue.tail = (*tcb).tcbEPPrev as usize;
        }
        queue
    }
}

pub fn Arch_initContext(mut context: arch_tcb_t) -> arch_tcb_t {
    (context).registers[SSTATUS] = 0x00040020;
    context
}
