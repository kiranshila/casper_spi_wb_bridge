use casper_spi_wb_bridge::SpiWbBridge;
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

// SNAP board uses Spi0, Ss0, and Mode0

fn main() {
    let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 4_000_000, Mode::Mode0)
        .expect("RPI SPI aquisition failed");
    let mut bridge = SpiWbBridge::new(spi);
    bridge
        .write(
            0x37184,
            u32::from_le_bytes("buhh".as_bytes().try_into().unwrap()),
        )
        .unwrap();
    println!(
        "{}",
        std::str::from_utf8(
            &bridge
                .read(0x37184)
                .expect("SPI reading failed")
                .to_ne_bytes()
        )
        .expect("Invalid UTF8")
    );
}
