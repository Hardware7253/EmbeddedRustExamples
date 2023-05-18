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
// VCC - 5V
// QA - 1602 DB0
// SER - pb12 (serial data)
// !OE - GND
// RCLK - pb13 (data latch)
// SRCLK - pb14 (serial clock)
// !SRCLR - 5V
// QH' - No connection

// 1602 Witing
// https://www.openhacks.com/uploadsproductos/eone-1602a1.pdf
//
// VSS - GND
// VDD - 5V
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
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

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

    // Initialise pins
    // When initializing pins use crl register for pins 0-7 and crh for pins 8-15
    let mut lcd = Lcd {
        serial_clock: gpiob.pb14.into_push_pull_output(&mut gpiob.crh).downgrade(),
        serial_data: gpiob.pb12.into_push_pull_output(&mut gpiob.crh).downgrade(),
        latch: gpiob.pb13.into_push_pull_output(&mut gpiob.crh).downgrade(),
        register_select: gpiob.pb15.into_push_pull_output(&mut gpiob.crh).downgrade(),
    };
    lcd.init(&mut delay);  // Initialise display

    lcd.print(&mut delay, "     Hello");
    lcd.set_cursor(&mut delay, [0, 1]);
    lcd.print(&mut delay, "     World");

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

struct Lcd {
    serial_clock: Pxx<Output<PushPull>>,
    serial_data: Pxx<Output<PushPull>>,
    latch: Pxx<Output<PushPull>>,
    register_select: Pxx<Output<PushPull>>,
}


// Instructions derived from various datasheets
// https://www.sparkfun.com/datasheets/LCD/ADM1602K-NSW-FBS-3.3v.pdf
// https://www.openhacks.com/uploadsproductos/eone-1602a1.pdf
// Non 1602 lcds might have different instructions, and definitely different ddram addresses
impl Lcd {

    // Writes a byte to a character lcd, data_input sets register select pin
    fn write(&mut self, delay: &mut Delay, data_input: bool, data: u8) {
        digital_write(&mut self.register_select, data_input); // Set data_input / instruction input
        
        shift_out(&mut self.serial_data, &mut self.serial_clock, delay, 8, data as u64, true);

        // Latch serial data to 74HC595 outputs, and read data on the 1602 data bus
        // This happens simultaneously becuase the latch pins are connected
        pulse_pin(&mut self.latch, delay, 1);

        delay.delay_ms(1u32); // Ensure there is time inbetween character lcd writes
    }

    // Initialze character lcd
    fn init(&mut self, delay: &mut Delay) {
        self.serial_clock.set_low().ok();
        self.latch.set_low().ok();

        self.write(delay, false, 0b00111000); // Initialize lcd with 8-bit bus, 2 lines, and 5x8 dot format

        self.clear(delay); // Clear dispaly
        self.home(delay);  // Home cursor
        self.power(delay, true, false, false); // Power on display, and hide the cursor
    }

    // Turn on/off display, cursor, and cursor position
    fn power(&mut self, delay: &mut Delay, display_on: bool, cursor_on: bool, cursor_position_on: bool) {
        let mut write_byte: u8 = 8;

        if display_on {
            write_byte += 4;
        }

        if cursor_on {
            write_byte += 2;
        }

        if cursor_position_on {
            write_byte += 1;
        }

        self.write(delay, false, write_byte);
    }

    // Clear display
    fn clear(&mut self, delay: &mut Delay) {
        self.write(delay, false, 0b00000001);
    }

    // Home cursor
    fn home(&mut self, delay: &mut Delay) {
        self.write(delay, false, 0b00000010);
    }

    // Shift cursor/display once in the specified direction
    fn shift(&mut self, delay: &mut Delay, shift_display: bool, shift_right: bool) {
        let mut write_byte: u8 = 0b00010000;

        if shift_display {
            write_byte += 0b00001000;
        }

        if shift_right {
            write_byte += 0b00000100;
        }

        self.write(delay, false, write_byte);
    }

    // Sets ddram (cursor) address
    fn set_ddram(&mut self, delay: &mut Delay, ddram_address: u8) {
        let mut write_byte: u8 = 0b10000000;
        write_byte ^= ddram_address;

        self.write(delay, false, write_byte);
    }

    // Sets the cursor position with cartesian coordinates
    fn set_cursor(&mut self, delay: &mut Delay, new_position: [u8; 2]) {
        // Character lcds that aren't a 1602 will have different and more/less ddram addresses
        let ddram_addresses: [[u8; 16]; 2] = [
            [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F],
            [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F],
        ];
        
        let address = ddram_addresses[new_position[1] as usize][new_position[0] as usize];
        self.set_ddram(delay, address);
    }

    // Prints a string to the lcd
    fn print(&mut self, delay: &mut Delay, string: &str) {
        for c in string.chars() {
            self.write(delay, true, c as u8);
        }
    }
}
