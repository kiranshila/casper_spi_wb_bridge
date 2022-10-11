use anyhow::{bail, Result};
use embedded_hal::blocking::spi;
use packed_struct::prelude::*;
use thiserror::Error;

#[derive(PackedStruct, Default, Debug)]
#[packed_struct(size_bytes = "14", bit_numbering = "msb0")]
pub struct Write {
    #[packed_field(bits = "0")]
    _write: ReservedOne<packed_bits::Bits<1>>,
    #[packed_field(bits = "1..=3")]
    _write_zeros: ReservedZero<packed_bits::Bits<3>>,
    #[packed_field(bits = "4..=7")]
    byte_enable: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bytes = "1..=4", endian = "lsb")]
    addr: u32,
    #[packed_field(bytes = "5..=8", endian = "lsb")]
    data: u32,
    #[packed_field(bits = "72..=111")]
    _zeros: ReservedZero<packed_bits::Bits<40>>,
}

#[derive(PackedStruct, Default, Debug)]
#[packed_struct(size_bytes = "14", bit_numbering = "msb0")]
pub struct Read {
    #[packed_field(bits = "0..=3")]
    _read_zeros: ReservedZero<packed_bits::Bits<4>>,
    #[packed_field(bits = "4..=7")]
    byte_enable: Integer<u8, packed_bits::Bits<4>>,
    #[packed_field(bytes = "1..=4", endian = "lsb")]
    addr: u32,
    #[packed_field(bits = "40..=111")]
    _zeros: ReservedZero<packed_bits::Bits<72>>,
}

#[derive(PackedStruct, Default, Debug)]
#[packed_struct(size_bytes = "14", bit_numbering = "msb0")]
pub struct Response {
    #[packed_field(bits = "0..=71")]
    _dummy: ReservedZero<packed_bits::Bits<72>>,
    #[packed_field(bits = "72..=103", endian = "lsb")]
    read_data: u32,
    #[packed_field(bits = "104")]
    ack: bool,
    #[packed_field(bits = "105")]
    error: bool,
    #[packed_field(bits = "106..=107")]
    _zeros: ReservedZero<packed_bits::Bits<2>>,
    #[packed_field(bits = "108..=111")]
    byte_enable: Integer<u8, packed_bits::Bits<4>>,
}

impl Write {
    fn new(byte_enable: u8, addr: u32, data: u32) -> Self {
        Self {
            byte_enable: byte_enable.into(),
            addr,
            data,
            ..Default::default()
        }
    }
}

impl Read {
    fn new(byte_enable: u8, addr: u32) -> Self {
        Self {
            byte_enable: byte_enable.into(),
            addr,
            ..Default::default()
        }
    }
}

/// The state for the SPI Wishbone Bridge
pub struct SpiWbBridge<SPI> {
    spi: SPI,
}

impl<SPI> SpiWbBridge<SPI>
where
    SPI: spi::Write<u8> + spi::Transfer<u8>,
{
    /// Construct a new instance of the bridge, given an SPI instance
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Given a u32 wishbone address, get the 4 byte word data at that address
    pub fn read(&mut self, addr: u32) -> Result<[u8; 4]> {
        let mut payload = [0u8; 14];
        Read::new(0b1111, addr).pack_to_slice(&mut payload)?;
        let resp_payload = match self.spi.transfer(&mut payload) {
            Ok(x) => x,
            Err(_) => bail!(Error::BadWrite),
        };
        let resp = Response::unpack_from_slice(resp_payload)?;
        if !resp.ack {
            bail!(Error::MissingAck);
        }
        if resp.error {
            bail!(Error::PayloadError);
        }
        Ok(resp.read_data.to_ne_bytes())
    }

    /// Write a 4 byte word `data` at the 32 bit wishbone address
    pub fn write(&mut self, addr: u32, data: &[u8; 4]) -> Result<()> {
        let mut payload = [0u8; 14];
        Write::new(0b1111, addr, u32::from_ne_bytes(*data)).pack_to_slice(&mut payload)?;
        let resp_payload = match self.spi.transfer(&mut payload) {
            Ok(x) => x,
            Err(_) => bail!(Error::BadWrite),
        };
        let resp = Response::unpack_from_slice(resp_payload)?;
        if !resp.ack {
            bail!(Error::MissingAck);
        }
        if resp.error {
            bail!(Error::PayloadError);
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
/// The errors we can return from interacting with the wishbone bridge
pub enum Error {
    #[error("Response has invalid ack")]
    MissingAck,
    #[error("SPI write errored")]
    BadWrite,
    #[error("Response error bit was true")]
    PayloadError,
}
