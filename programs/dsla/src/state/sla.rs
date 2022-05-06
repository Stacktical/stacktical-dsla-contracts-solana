use crate::errors::ErrorCode;
use anchor_lang::prelude::*;

#[account]
pub struct Sla {
    pub messenger_address: Pubkey,
    pub slo: Slo,
    pub leverage: u64,
    pub ipfs_hash: String,
    pub mint_address: Pubkey,
}

impl Sla {
    // discriminator + messenger_address + SLO + leverage + ipfs_hash + mint + authority + mint_address
    pub const LEN: usize = 8 + 32 + Slo::LEN + 8 + 32 + 32 + 32 + 16 + 16 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Slo {
    pub slo_value: u128,
    pub slo_type: SloType,
}

impl Slo {
    /// slo_value + slo_type
    pub const LEN: usize = 16 + 1;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn get_deviation_invalid_precision_1() {
        let slo = Slo {
            slo_value: 10000,
            slo_type: SloType::EqualTo,
        };

        slo.get_deviation(5000, 10).unwrap();
    }
    #[test]
    #[should_panic]
    fn get_deviation_invalid_precision_2() {
        let slo = Slo {
            slo_value: 100000,
            slo_type: SloType::NotEqualTo,
        };

        slo.get_deviation(5000, 100001).unwrap();
    }
}
