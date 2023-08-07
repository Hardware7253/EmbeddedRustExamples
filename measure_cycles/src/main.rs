// This program prints the current elapsed cpu cycles every second

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal as hal;
use hal::{pac, pac::DWT, pac::DCB, delay::Delay, prelude::*};

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    // Init buffers for debug printing
    rtt_init_print!();

    // Get access to device and core peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // Get access to RCC, FLASH and GPIOC
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();

    // Configure and apply clock configuration
    let clock_mhz = 48;
    let clocks = rcc.cfgr
        // External oscillator
        .use_hse(16.mhz())

        // Bus and core clocks
        .hclk(clock_mhz.mhz())
        .sysclk(clock_mhz.mhz())

        // Peripheral clocks
        .pclk1(12.mhz())
        .pclk2(12.mhz())
    .freeze(&mut flash.acr);
    
    // Set up systick delay
    let mut delay = Delay::new(cp.SYST, clocks);

    // Enable cycle counter
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    let mut cycle_resets: u32 = 0;
    let mut last_cycle_count: u32 = 0;
    
    loop {
        
        // Whenever the DWT cycle count resets increment cycle_resets
        let current_cycles = DWT::cycle_count();
        if current_cycles < last_cycle_count {
            cycle_resets += 1; 
        }
        last_cycle_count = current_cycles;

        // Because the DWT cycle_count is a u32 it resets frequently due to the high 48Mhz clockspeed
        // Keeping track of the cycle resets can be used to store the cycles as a u64
        // The u64 value will take thousands of years to reset
        let cycles: u64 = (cycle_resets as u64 * u32::MAX as u64) + current_cycles as u64;

        rprintln!("{}", cycles);
        delay.delay_ms(1000u16);
    }
}
