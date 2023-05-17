// This program shifts a number into a 74HC595 shift register and latches the output

// Wiring
// https://www.ti.com/lit/ds/symlink/sn74hc595.pdf?ts=1684225092432&ref_url=https%253A%252F%252Fwww.ti.com%252Fproduct%252FSN74HC595
//
// QB - Output
// QC - Output
// QD - Output
// QE - Output
// QF - Output
// QG - Output
// QH - Output
// GND - GND
//
// VCC - VCC
// QA - Output
// SER - pb12
// !OE - GND
// RCLK - pb13
// SRCLK - pb14
// !SRCLR - VCC
// QH' - No connection

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal as hal;
use embedded_hal::digital::v2::OutputPin;
use hal::{pac, delay::Delay, prelude::*};
use hal::gpio::{Pxx, PushPull, Output};

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
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // Initialze shift register pins
    // When initializing pins use crl register for pins 0-7 and crh for pins 8-15
    let mut clock = gpiob.pb14.into_push_pull_output(&mut gpiob.crh).downgrade();
    let mut latch = gpiob.pb13.into_push_pull_output(&mut gpiob.crh).downgrade();
    let mut data = gpiob.pb12.into_push_pull_output(&mut gpiob.crh).downgrade();

    clock.set_low().ok();
    latch.set_low().ok();

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

    delay.delay_us(1u32);

    let shift_register_bits = 8;

    // Shiftout data
    shift_out(&mut data, &mut clock, &mut delay, shift_register_bits, 11, true);

    delay.delay_us(1u32);

    // Latch data
    latch.set_high().ok();
    delay.delay_us(1u32);
    latch.set_low().ok();
    
    loop {
        
    }
}

// Shifts given number into shift register
fn shift_out(data_pin: &mut Pxx<Output<PushPull>>, clock_pin: &mut Pxx<Output<PushPull>>, delay: &mut Delay, bits: usize, num: u64, msbfirst: bool) {
    for i in 0..bits {
        
        if !msbfirst {
            digital_write(data_pin, bit_on(num, i)); 
        } else {
            digital_write(data_pin, bit_on(num, (i as i16 - (bits as i16 - 1)).abs().try_into().unwrap())); 
        }
        
        delay.delay_us(1u32);

        clock_pin.set_high().ok();
        delay.delay_us(1u32);
        clock_pin.set_low().ok();
        delay.delay_us(1u32);
    }
}

// Returns true if a bit is on in a u64 number
pub fn bit_on(num: u64, bit: usize) -> bool {
    let num_from_bit = 1 << bit;
    if num ^ num_from_bit < num {
        return true;
    }
    false
}

// Writes low or high state to given pin
fn digital_write(pin: &mut Pxx<Output<PushPull>>, high: bool) {
    if high {
        pin.set_high().ok();
        return;
    }
    pin.set_low().ok();
}
