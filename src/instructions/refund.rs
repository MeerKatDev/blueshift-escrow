use pinocchio::ProgramResult;
use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;

pub struct Refund {}

impl<'a> Refund {
    pub const DISCRIMINATOR: &'a u8 = &3;

    pub fn process(&self) -> ProgramResult {
        Ok(())
    }
}

impl<'a> TryFrom<&'a [AccountInfo]> for Refund {
    type Error = ProgramError;

    fn try_from(_accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        Ok(Self {})
    }
}