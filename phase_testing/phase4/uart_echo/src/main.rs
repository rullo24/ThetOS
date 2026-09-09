#![no_std]
#![no_main]

// STD INCLUDES
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};

// USER INCLUDES
use nucleo_l152re::{Serial, System, TaskId, TaskPriority, Uart, UartConfig, UninitUart, PA2, PA3};
use thetos_entry::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 4096] = [0; 4096];

// echo every received byte back over USART2 (the ST-LINK virtual COM port)
extern "C" fn echo_task(_arg: *mut ()) -> ! {
    let mut serial = Serial::usart2(PA2, PA3).into_active(UartConfig::default());

    serial.write(b"uart echo ready\r\n");

    loop {
        match serial.read_byte() {
            Ok(Some(byte)) => serial.write_byte(byte),
            Ok(None) => {}
            Err(_) => serial.write_byte(b'?'), // overrun / framing / parity
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    system
        .spawn_task(
            TaskId(1),
            TaskPriority::default(),
            2048,
            echo_task,
            null_mut(),
        )
        .unwrap();

    system.run();
}
