//uint256 internal constant Q128 = 0x100000000000000000000000000000000;
//uint8 internal constant FP96_RESOLUTION = 96;
//uint256 internal constant Q96 = 0x1000000000000000000000000;

use primitive_types::U256;

pub const FP96_RESOLUTION: u8 = 96;

lazy_static! {
    pub static ref q128: U256 = U256::from_big_endian(&[01,00,00,00,00,00,00,00,00,00,00,00,00,00,00,00,00]); //wow ugh
    pub static ref q96: U256 = U256::from_big_endian(&[01,00,00,00,00,00,00,00,00,00,00,00,00]); //wow ugh #2
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        unimplemented!()
    }
}
