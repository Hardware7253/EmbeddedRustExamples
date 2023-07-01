# EmbeddedRustExamples
Example programs for embedded rust, specifically for the STM32F103C8T6 (bluepill) microcontroller.
Different installation steps and configuration may be required for different STM32 microcontrollers, especially if they have a different architecture.

The default clock configuration may need to be changed depending on your STM32F103C8T6 board.
The default clock configuration in each example project uses a 16Mhz external clock, with a 48Mhz core clock.

## Install Instructions
1. Ensure rust and rustup are installed on your system
2. `rustup target add thumbv7m-none-eabi` Add Cortex-M3 target platform
3. `cargo install cargo-binutils` Dependency for next step, ensure build-essential is installed on your system first
4. `rustup component add llvm-tools-preview` Binutils alternative for all supported rust architectures
5. `cargo install cargo-flash` Install cargo-flash for flashing the microcontroller, ensure libusb is installed first
6. `cargo install cargo-embed` Install cargo-embed for easy compiling, uploading, and rtt debugging
7. Setup udev rules so you have access to the probe [Instructions](https://probe.rs/docs/getting-started/probe-setup/)

## Usage Instructions
All the configuration has allready been done inside of each projects `Cargo.toml`, `Embed.toml`, `memory.x` and `.cargo/config` files for the STM32F103C8T6.
The project can be flashed to a swd probe using `cargo flash --chip stm32f103c8t6 --release`, or can be flashed and debugged using `cargo embed --release`
