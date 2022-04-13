use anchor_lang::prelude::*;

use crate::errors::ErrorCode;

#[account]
pub struct Slo {
    pub slo_value: u128,
    pub slo_type: SloType,
    pub bump: u8,
}

impl Slo {
    // slo_value + slo_type + bump
    pub const MAX_SIZE: usize = 16 + 1 + 1;

    pub fn is_respected(&self, value: u128) -> Result<bool> {
        let slo_type = self.slo_type;
        let slo_value = self.slo_value;

        match slo_type {
            SloType::EqualTo => Ok(value == slo_value),
            SloType::NotEqualTo => Ok(value != slo_value),
            SloType::SmallerThan => Ok(value < slo_value),
            SloType::SmallerOrEqualTo => Ok(value <= slo_value),
            SloType::GreaterThan => Ok(value > slo_value),
            SloType::GreaterOrEqualTo => Ok(value >= slo_value),
        }
    }

    pub fn get_deviation(&self, sli: u128, precision: u128) -> Result<u128> {
        if (precision % 100 != 0) || (precision == 0) {
            return err!(ErrorCode::InvalidPrecision);
        }

        let slo_type = self.slo_type;
        let slo_value = self.slo_value;

        let mut deviation: u128 = (if sli >= slo_value {
            sli - slo_value
        } else {
            slo_value
        }) * precision
            / ((sli + slo_value) / 2);

        if deviation > (precision * 25 / 100) {
            deviation = precision * 25 / 100;
        }
        match slo_type {
            // Deviation of 1%
            SloType::EqualTo | SloType::NotEqualTo => Ok(precision / 100),
            _ => Ok(deviation),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum SloType {
    EqualTo,
    NotEqualTo,
    SmallerThan,
    SmallerOrEqualTo,
    GreaterThan,
    GreaterOrEqualTo,
}
