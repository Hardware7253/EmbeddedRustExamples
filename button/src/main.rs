// This program reads the state of a button, and prints that state
// A button can be connected with one side to PA9, and the other to +3.3V

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::InputPin;
use stm32f1xx_hal as hal;
use hal::{pac, delay::Delay, prelude::*};

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    // Init buffers for debug printing
    rtt_init_print!();

    // Get access to device and core peripherals
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // Get access to RCC, FLASH and GPIOC
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);

    // Initialize button pin
    // When initializing pins use crl register for pins 0-7 and crh for pins 8-15
    let button = gpioa.pa9.into_pull_down_input(&mut gpioa.crh).downgrade();

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
    
    loop {

        // Check if the button has been pressed
        if button.is_high().unwrap() {
            rprintln!("Button pressed");
        }
        delay.delay_ms(100u16);
    }
}
