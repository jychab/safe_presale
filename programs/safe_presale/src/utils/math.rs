use std::{cmp::Eq, convert::TryInto};
use uint::construct_uint;

use crate::error::CustomError;

construct_uint! {
    pub struct U128(2);
}

#[derive(Clone, Debug, PartialEq)]
pub struct Calculator {}

impl Calculator {
    pub fn to_u64(val: u128) -> Result<u64, CustomError> {
        val.try_into().map_err(|_| CustomError::ConversionFailure)
    }
    pub fn to_u64_from_i64(val: i64) -> Result<u64, CustomError> {
        val.try_into()
            .map_err(|_| CustomError::InvalidNegativeValue)
    }
}
