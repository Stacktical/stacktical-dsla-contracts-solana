use crate::errors::ErrorCode;
use crate::state::utils::{Add, Compare, Decimal, Div, Mul, Sub};
use anchor_lang::prelude::*;
#[account]
pub struct SlaAuthority {}
#[account]
pub struct Sla {
    /// address of the messeger providing the data
    pub messenger_address: Pubkey,
    /// service level objective, the objective to achieve for the provider to be rewarded
    pub slo: Slo,
    ///  leverage for the SLA between provider and user pool
    pub leverage: u64,
    pub ipfs_hash: String,
    /// address of the coin to be used as SLA reward for users and providers
    pub mint_address: Pubkey,
    /// The account derived by the program, which has authority over all
    /// assets in the SLA.
    pub sla_authority: Pubkey,
    /// The address used as the seed for generating the SLA authority
    /// address. Typically this is the SLA account's own address.
    pub authority_seed: Pubkey,
    /// The bump seed value for generating the authority address.
    pub authority_bump_seed: [u8; 1],
}

impl Sla {
    // discriminator + messenger_address + SLO + leverage + ipfs_hash + mint + authority + mint_address
    pub const LEN: usize = 8 + 32 + Slo::LEN + 8 + 32 + 32 + 32 + 16 + 16 + 32;
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct Slo {
    pub slo_value: Decimal,
    pub slo_type: SloType,
}

impl Slo {
    /// slo_value + slo_type
    pub const LEN: usize = 16 + 1;

    pub fn is_respected(&self, value: Decimal) -> Result<bool> {
        let slo_type = self.slo_type;
        let slo_value = self.slo_value;

        match slo_type {
            SloType::EqualTo => Ok(value.eq(slo_value).unwrap()),
            SloType::NotEqualTo => Ok(value.ne(&slo_value)),
            SloType::SmallerThan => Ok(value.lt(slo_value).unwrap()),
            SloType::SmallerOrEqualTo => Ok(value.lte(slo_value).unwrap()),
            SloType::GreaterThan => Ok(value.gt(slo_value).unwrap()),
            SloType::GreaterOrEqualTo => Ok(value.gte(slo_value).unwrap()),
        }
    }

    pub fn get_deviation(&self, sli: Decimal, precision: u128) -> Result<Decimal> {
        if (precision % 100 != 0) || (precision == 0) {
            return err!(ErrorCode::InvalidPrecision);
        }
        let precision = Decimal::new(precision, 0);
        let slo_type = self.slo_type;
        let slo_value = self.slo_value;

        let mut deviation: Decimal = (if sli.gte(slo_value).unwrap() {
            sli.sub(slo_value).unwrap()
        } else {
            slo_value
        })
        .mul(precision)
        .div((sli.add(slo_value)).unwrap().div(Decimal::new(2, 0)));

        if deviation
            .gt(precision
                .mul(25)
                .div(Decimal::new(100, 0))
                .to_scale(deviation.scale))
            .unwrap()
        {
            deviation = precision
                .mul(25)
                .div(Decimal::new(100, 0))
                .to_scale(deviation.scale);
        }
        match slo_type {
            // Deviation of 1%
            SloType::EqualTo | SloType::NotEqualTo => Ok(precision.div(Decimal::new(100, 0))),
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
