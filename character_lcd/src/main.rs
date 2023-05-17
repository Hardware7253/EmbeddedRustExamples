// This program controls a 1602 character lcd with a 74HC595 shift register

// 74HC595 Wiring
// https://www.ti.com/lit/ds/symlink/sn74hc595.pdf?ts=1684225092432&ref_url=https%253A%252F%252Fwww.ti.com%252Fproduct%252FSN74HC595
//
// QB - 1602 DB1
// QC - 1602 DB2
// QD - 1602 DB3
// QE - 1602 DB4
// QF - 1602 DB5
// QG - 1602 DB6
// QH - 1602 DB7
// GND - GND
//
// VCC - VCC
// QA - 1602 DB0
// SER - pb12 (serial data)
// !OE - GND
// RCLK - pb13 (data latch)
// SRCLK - pb14 (serial clock)
// !SRCLR - VCC
// QH' - No connection

// 1602 Witing
// https://www.openhacks.com/uploadsproductos/eone-1602a1.pdf
//
// GND - GND
// VCC - VCC
// VO - Potentiometer (for contrast)
// RS - (register select)
// R/W - GND (always write)
// E - pb13 (shared latch with 74HC595)
// Data lines connected to 7HC595

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f1xx_hal as hal;
use embedded_hal::digital::v2::OutputPin;
use hal::{pac, delay::Delay, prelude::*};
use hal::gpio::{Pxx, PushPull, Output};

use rtt_target::{rprintln, rtt_init_print};

struct LcdPins {
    serial_clock: Pxx<Output<PushPull>>,
    serial_data: Pxx<Output<PushPull>>,
    latch: Pxx<Output<PushPull>>,
    register_select: Pxx<Output<PushPull>>,
}

impl LcdPins {
    fn init(&mut self) {
        self.serial_clock.set_low().ok();
        self.latch.set_low().ok();
    }  
}

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

    // Initialze pins
    // When initializing pins use crl register for pins 0-7 and crh for pins 8-15
    let mut lcd_pins = LcdPins {
        serial_clock: gpiob.pb14.into_push_pull_output(&mut gpiob.crh).downgrade(),
        serial_data: gpiob.pb12.into_push_pull_output(&mut gpiob.crh).downgrade(),
        latch: gpiob.pb13.into_push_pull_output(&mut gpiob.crh).downgrade(),
        register_select: gpiob.pb15.into_push_pull_output(&mut gpiob.crh).downgrade(),
    };
    lcd_pins.init();

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
        
    }
}

// Writes a byte to a character lcd, data_input sets register select pin
fn write_lcd(pins: &mut LcdPins, delay: &mut Delay, data_input: bool, data: u8) {
    digital_write(&mut pins.register_select, data_input); // Set data_input / instruction input
    
    shift_out(&mut pins.serial_data, &mut pins.serial_clock, delay, 8, data as u64, true);

    let latch = &mut pins.latch;

    // Latch serial data to 74HC595 outputs
    pulse_pin(latch, delay, 1);

    // Latch 74HC595 outputs to 1602 data pins
    pulse_pin(latch, delay, 1);
}

// Shifts given number into shift register
fn shift_out(data_pin: &mut Pxx<Output<PushPull>>, clock_pin: &mut Pxx<Output<PushPull>>, delay: &mut Delay, bits: usize, num: u64, msbfirst: bool) {
    for i in 0..bits {
        
        if !msbfirst {
            digital_write(data_pin, bit_on(num, i)); 
        } else {
            digital_write(data_pin, bit_on(num, (i as i16 - (bits as i16 - 1)).abs().try_into().unwrap())); 
        }
        
        delay.delay_us(1u32); // Data hold time

        pulse_pin(clock_pin, delay, 1); // Pulse clock for 1 micro second
    }
}

// Sets a pin high, waits micro_seconds, sets the pin low, waits micro_seconds
fn pulse_pin(pin: &mut Pxx<Output<PushPull>>, delay: &mut Delay, micro_seconds: u32) {
    pin.set_high().ok();
    delay.delay_us(micro_seconds);
    pin.set_low().ok();
    delay.delay_us(micro_seconds);
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
