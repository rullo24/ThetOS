/// GREAT REFERENCE: https://github.com/FreeRTOS/FreeRTOS-Kernel/blob/main/portable/GCC/ARM_CM3/port.c

/// imports
use core::arch::global_asm;
use core::ptr::{addr_of_mut, write_volatile};
use super::v7m_exception_frame::V7mHwExceptionFrame;

// REFERENCE: https://interrupt.memfault.com/blog/cortex-m-rtos-context-switching
// const EXC_RETURN_MSP_HANDLER_MODE: u32 = 0xFFFFFFF1;
const EXC_RETURN_PSP_THREAD_MODE: u32 = 0xFFFFFFFD;
const EXC_RETURN_MSP_THREAD_MODE: u32 = 0xFFFFFFF9;

// REFERENCE: https://interrupt.memfault.com/blog/cortex-m-rtos-context-switching
/// EXC_RETURN[2]: 0 == stacked frame on MSP; 1 == on PSP (CONTROL.SPSEL is not valid inside Handler mode)
const EXC_RETURN_THREAD_STACK_MASK: u32 = EXC_RETURN_PSP_THREAD_MODE ^ EXC_RETURN_MSP_THREAD_MODE;
const _: () = assert!(EXC_RETURN_THREAD_STACK_MASK == 1 << 2); // comptime check for THREAD MODE (no MSP Handler)

// REFERENCE: https://developer.arm.com/documentation/dui0552/a/the-cortex-m3-processor/programmers-model/core-registers?lang=en
/// CONTROL.SPSEL[1] -> 0 == Thread used MSP before; 1 == Thread used PSP before
// const CONTROL_SPSEL_MASK: u32 = 1 << 1; // Thread used PSP before

/// auto-stacked exception frame size (R0..xPSR)
const V7M_HW_EXCEPTION_FRAME_BYTES: u32 = V7mHwExceptionFrame::FRAME_SIZE_BYTES as u32;

/// next task PSP (frame base ptr from `initialise_task_context`) -> must be set before requesting PendSV
#[no_mangle] 
static mut PENDSV_NEXT_PSP: u32 = 0; // global -> used in asm code (label must be available as printed)

/// current task TCB (from `get_current_task`) -> must be set before requesting PendSV
#[no_mangle]
static mut PENDSV_CURRENT_TASK: *mut super::V7mTaskContext = core::ptr::null_mut();

/// TEMP DIAGNOSTIC (#41): counts how many times the CPU actually takes PendSV, incremented by the handler itself
#[no_mangle]
pub static mut PENDSV_ENTRY_COUNT: u32 = 0;

/// DESCRIPTION
/// set the PSP value the PendSV handler will restore -> must match `V7mTaskContext.sp`
pub unsafe fn set_next_task_psp(psp: *mut u8) {
    write_volatile(addr_of_mut!(PENDSV_NEXT_PSP), psp as usize as u32);
}

/// DESCRIPTION
/// set the outgoing task for PSP-thread saves -> null skips storing ongoing SP (i.e. first switch from MSP)
pub unsafe fn set_current_task_tcb(tcb: *mut super::V7mTaskContext) {
    write_volatile(addr_of_mut!(PENDSV_CURRENT_TASK), tcb);
}

// DESCRIPTION
// PendSV_Handler -> called by hardware when PendSV is taken
global_asm!(
    ".syntax unified",
    ".global PendSV_Handler", // can be referenced from other files
    ".type PendSV_Handler, %function", // function type -> used by linker to place in correct section
    "PendSV_Handler:", // start of PendSV_Handler label (func)

    // TEMP DIAGNOSTIC (#41): count every actual entry, r0/r1 safe to clobber here (overwritten below regardless)
    "ldr r0, ={entry_count}",
    "ldr r1, [r0]",
    "adds r1, r1, #1",
    "str r1, [r0]",

    // 1) Which stack did Thread mode use? (CONTROL.SPSEL) -> where the HW stacked the 8-word frame.
    "tst lr, {exc_return_stack}", // Z=1 if frame on MSP (e.g. 0xFFFFFFF9); Z=0 if on PSP (e.g. 0xFFFFFFFD)
    "bne .L_pendsv_thread_uses_psp", // if SPSEL=1 (branch when Z is 0), frame is on PSP -> save outgoing task

    // 2a) Thread used MSP: HW frame is on MSP -> discard it (we do not return to that thread here).
    ".L_pendsv_thread_uses_msp:", // not jumped to -> here for readability
    "mrs r0, msp", // r0 reads in MSP
    "adds r0, r0, {hw_frame_bytes}", // discard 8-word hardware frame -> moves r0 to the base of the stack frame (deallocs)
    "msr msp, r0", // update MSP to point to new "top" of stack (hold MSP exception frame in addr under this)
    "b .L_pendsv_schedule", // skip PSP save -> MSP save complete

    // 2b) Thread used PSP: HW stacked R0..xPSR on PSP; save callee regs the HW does not.
    ".L_pendsv_thread_uses_psp:",
    "mrs r0, psp", // r0 reads in PSP
    "isb", // flush asm pipeline to ensure PSP is synchronised before stack access
    "stmdb r0!, {{r4-r11}}", // ('r0!' updates r0 w/ final addr after store finished) | takes addr of r0 and decreases it by 4-bytes for ea register + store contents of R4-R11 into new mem locations
    "ldr r3, ={current_task}", // load addr of current task into r3
    "ldr r2, [r3]", // deref val in r3 and store into r2
    "cmp r2, #0", // compare r2 with NULL (0x0) -> written to APSR
    "beq .L_pendsv_schedule", // branch if r2 == 0x0 (saved in Z flag of APSR)
    "str r0, [r2]", // write new PSP addr into r2

    // 3) Scheduler hook
    ".L_pendsv_schedule:",
    "nop", // Placeholder -> any `bl` needs EXC_RETURN saved on MSP first

    // 4) Restore incoming task -> `PENDSV_NEXT_PSP` must match `V7mTaskContext.sp` (callee + HW layout).
    ".L_pendsv_restore:",
    "ldr r3, ={next_psp}", // load addr of next PSP into r3
    "ldr r0, [r3]", // load val at r3 into r0
    "ldmia r0!, {{r4-r11}}", // pop callee-saved regs -> r0 addr moves 'up' to the hardware frame base
    "msr psp, r0", // move r0 into PSP (new stack lowest addr)
    "isb", // flush asm pipeline to ensure PSP is synchronised
    "ldr lr, ={exc_return_psp}", // EXC_RETURN set to PSP thread mode to return to new task
    "bx r14", // trigger exception return -> HW pops hardware frame from PSP and resumes new task

    // labels that are replaced at compile time
    // control_spsel = const CONTROL_SPSEL_MASK,
    exc_return_stack = const EXC_RETURN_THREAD_STACK_MASK,
    hw_frame_bytes = const V7M_HW_EXCEPTION_FRAME_BYTES,
    current_task = sym PENDSV_CURRENT_TASK,
    next_psp = sym PENDSV_NEXT_PSP,
    exc_return_psp = const EXC_RETURN_PSP_THREAD_MODE,
    entry_count = sym PENDSV_ENTRY_COUNT,
);