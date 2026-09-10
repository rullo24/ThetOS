#![no_std]
#![no_main]

// STD INCLUDES
use core::panic::PanicInfo;
use core::ptr::{addr_of_mut, null_mut};

// USER INCLUDES
use nucleo_l152re::{
    system, Pwm, PwmChannel, PwmConfig, System, TaskId, TaskPriority, Tim2, UninitPwm, PA5,
};
use thetos_entry::entry;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static mut STACK_POOL: [u8; 4096] = [0; 4096];

const FADE_STEPS: u16 = 64; // duty points per ramp
const STEP_MS: u32 = 15; // dwell at each point -> ~1s per ramp, ~2s breath

// map a ramp index onto the full u16 duty range
fn step_duty(index: u16) -> u16 {
    ((index as u32 * u16::MAX as u32) / FADE_STEPS as u32) as u16
}

// breathe LD2 (PA5) with TIM2_CH1 hardware PWM
extern "C" fn fade_task(_arg: *mut ()) -> ! {
    let pwm = Pwm::<Tim2, _>::new().into_active(PwmConfig { freq_hz: 1_000 });
    let mut led = pwm.channel1(PA5);
    led.enable();

    loop {
        for index in 0..=FADE_STEPS {
            led.set_duty(step_duty(index));
            system::delay_ms(STEP_MS).unwrap();
        }
        for index in (0..=FADE_STEPS).rev() {
            led.set_duty(step_duty(index));
            system::delay_ms(STEP_MS).unwrap();
        }
    }
}

#[entry]
fn app_main() -> ! {
    let p_stack_pool = unsafe { &mut *addr_of_mut!(STACK_POOL) };
    let mut system = System::new_with_pool(p_stack_pool).unwrap();

    system
        .spawn_task(TaskId(1), TaskPriority::default(), 2048, fade_task, null_mut())
        .unwrap();

    system.run();
}
