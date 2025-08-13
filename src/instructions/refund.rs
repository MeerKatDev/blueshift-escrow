use crate::accounts::{AccountCheck, AccountClose};
use crate::{AssociatedTokenAccount, Escrow, MintInterface, ProgramAccount, SignerAccount};
use pinocchio::account_info::AccountInfo;
use pinocchio::instruction::{Seed, Signer};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::create_program_address;
use pinocchio::ProgramResult;
use pinocchio_token::instructions::{CloseAccount, Transfer};
use pinocchio_token::state::TokenAccount;


/// maker: the creator of the escrow. Must be a signer and mutable
/// escrow: the escrow account that we're initializing. Must be mutable
/// mint_a: the token we're depositing in the escrow
/// vault: the associated token account owned by the escrow. Must be mutable
/// maker_ata_a: the associated token account owned by the maker. Must be mutable
/// system_program: the system program. Must be executable
/// token_program: the token program. Must be executable
pub struct RefundAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Basic Accounts Checks
        SignerAccount::check(maker)?;
        ProgramAccount::check(escrow)?;
        MintInterface::check(mint_a)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;

        // Return the accounts
        Ok(Self {
            maker,
            escrow,
            mint_a,
            vault,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        // Initialize necessary accounts
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

/// The refund instruction lets the maker cancel an open offer:
///
/// Close the escrow PDA and send its rent lamports back to the maker.
/// Move the full Token A balance out of the vault and back to the maker, 
/// then close the vault account.
impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        let (seed, bump, escrow_key) = {
            let data = self.accounts.escrow.try_borrow_data()?;
            let escrow = Escrow::load(&data)?;

            let key = create_program_address(
                &[
                    b"escrow",
                    self.accounts.maker.key(),
                    &escrow.seed.to_le_bytes(),
                    &escrow.bump,
                ],
                &crate::ID,
            )?;

            (escrow.seed, escrow.bump, key)
        };

        if &escrow_key != self.accounts.escrow.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        let seed_binding = seed.to_le_bytes();
        let bump_binding = bump;
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.key().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];
        let signer = Signer::from(&escrow_seeds);

        let vault_amount = {
            let v = TokenAccount::from_account_info(self.accounts.vault)?;
            v.amount()
        };

        Transfer {
            from: self.accounts.vault,
            to: self.accounts.maker_ata_a,
            authority: self.accounts.escrow,
            amount: vault_amount,
        }
        .invoke_signed(&[signer.clone()])?;

        // Close the Vault
        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }
        .invoke_signed(&[signer.clone()])?;

        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;

        Ok(())
    }
}
