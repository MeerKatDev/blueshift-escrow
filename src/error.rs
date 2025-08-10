use pinocchio::program_error::ProgramError;

pub enum PinocchioError {
    NotSigner,
    InvalidOwner,
    InvalidAccountData,
    InvalidAddress,
}

impl From<PinocchioError> for ProgramError {
    fn from(val: PinocchioError) -> Self {
        ProgramError::Custom(val as u32)
    }
}
