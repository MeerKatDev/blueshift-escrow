use pinocchio::program_error::ProgramError;

pub enum PinocchioError {
	NotSigner,
	InvalidOwner,
	InvalidAccountData,
	InvalidAddress
}

impl Into<ProgramError> for PinocchioError {
    fn into(self: PinocchioError) -> ProgramError {
        ProgramError::Custom(self as u32)
    }
}