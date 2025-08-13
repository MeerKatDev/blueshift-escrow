use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::ProgramResult;

pub struct Refund {}

impl Refund {
    pub const DISCRIMINATOR: &u8 = &2;

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
