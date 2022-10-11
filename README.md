# casper_spi_wb_bridge

[CASPER](https://casper.astro.berkeley.edu/) FPGA boards sometimes include an SPI
client that interfaces with its internal Wishbone bus. This rust library
provides a device-generic SPI driver that works with any
[embedded_hal](https://docs.rs/embedded-hal/latest/embedded_hal/) SPI device,
such as microcontrollers and embedded linux devices (like the Raspberry Pi).
