use core::ptr::{write_volatile, addr_of_mut};

// REFERENCE: arm developer -> Home / Documentation / IP Products / Processors / Cortex-M / Cortex-M3 / Cortex-M3 Devices Generic User Guide / Exception entry and return
// https://developer.arm.com/documentation/dui0552/a/the-cortex-m3-processor/exception-model/exception-entry-and-return?lang=en
// exception entry frame: R0-R3, R12, LR, PC, xPSR (ARMv7-M -> no FPU)
const V7M_NUM_BASIC_EXCEPTION_FRAME_REGISTERS: usize = 8;
const V7M_BASIC_EXCEPTION_FRAME_BYTES: usize = V7M_NUM_BASIC_EXCEPTION_FRAME_REGISTERS * core::mem::size_of::<u32>();

// thumb target tracking (bit[0]) -> for instructions entry/branch addresses that are funcs (runable rather than data)
pub const THUMB_TARGET_TRACKING_BIT: u32 = 1;

// layout pulled from reference above (SP+0x00 .. SP+0x1C)
#[repr(C)] // C-layout -> leave struct as is (no paddin)
pub struct V7mBasicExceptionFrame {
    pub r0: u32, // SP + 0x00
    pub r1: u32, // SP + 0x04
    pub r2: u32, // SP + 0x08
    pub r3: u32, // SP + 0x0C
    pub r12: u32, // SP + 0x10
    pub lr: u32, // SP + 0x14
    pub pc: u32, // SP + 0x18
    pub xpsr: u32, // SP + 0x1C
}

// ensuring that size of struct is as expected (8 * 4 bytes = 32 bytes)
const _: () = assert!(core::mem::size_of::<V7mBasicExceptionFrame>() == V7M_BASIC_EXCEPTION_FRAME_BYTES);

impl V7mBasicExceptionFrame {
    pub const FRAME_SIZE_BYTES: usize = core::mem::size_of::<Self>();

    // REFERENCE: arm developer -> Home / Documentation / IP Products / Processors / Cortex-M / Cortex-M3 / Cortex-M3 Devices Generic User Guide / Core registers
    // Table 2.6. EPSR bit assignments
    const EPSR_T_MASK: u32 = 1 << 24;
    // xPSR: IPSR=0 (thread mode), APSR=0 (no stale flags), EPSR.T=1 (Thumb mode); see Tables 2.4–2.6
    pub const INITIAL_XPSR: u32 = Self::EPSR_T_MASK;

    /// DESCRIPTION
    /// write the initial task frame registers into the exception frame
    pub unsafe fn write_initial_task_frame(
        frame_base: *mut u8, // SP -> base of the exception frame
        entry_point: extern "C" fn (*mut ()) -> !,
        entry_arg: *mut(),
        task_exit_lr: u32, // LR value to use for task exit (use v7m_default_task_exit)
    ) {

        // checking all ptrs are valid (not null)
        if frame_base.is_null() {
            panic!("frame_base is null");
        }
        if (entry_point as usize) == 0x0 {
            panic!("entry_point is null");
        }
        if task_exit_lr == 0x0 {
            panic!("task_exit_LR is null");
        }

        // cast the base ptr to a ptr to the exception frame (usable)
        let p_frame = &mut *(frame_base.cast::<V7mBasicExceptionFrame>()); 

        // write the task frame registers into block
        write_volatile(addr_of_mut!(p_frame.r0), entry_arg as usize as u32); // R0 = entry_arg (cast to u32)
        write_volatile(addr_of_mut!(p_frame.r1), 0x0); // R1 = 0x0
        write_volatile(addr_of_mut!(p_frame.r2), 0x0); // R2 = 0x0
        write_volatile(addr_of_mut!(p_frame.r3), 0x0); // R3 = 0x0
        write_volatile(addr_of_mut!(p_frame.r12), 0x0); // R12 = 0x0
        write_volatile(addr_of_mut!(p_frame.lr), task_exit_lr | THUMB_TARGET_TRACKING_BIT); // LR = v7m_default_task_exit (bitwise OR Thumb bit[0] so task exit runs as Thumb code)
        write_volatile(addr_of_mut!(p_frame.pc), entry_point as usize as u32 | THUMB_TARGET_TRACKING_BIT); // PC = entry_point (bitwise OR Thumb bit[0] so entry point runs as Thumb code)
        write_volatile(addr_of_mut!(p_frame.xpsr), Self::INITIAL_XPSR);
    }
}

/// DESCRIPTION
/// should never get here (this is a last resort catch to avoid breaking the kernel)
#[inline(never)] // ensure that the func is never copied (always pointer to this func)
pub extern "C" fn v7m_default_task_exit() -> ! {
    loop {
        core::hint::spin_loop(); 
    }
}