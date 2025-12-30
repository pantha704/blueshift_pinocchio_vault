use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

// Structure to hold the accounts for the Withdraw instruction.
// Pinocchio requires manual definition and parsing of accounts.
pub struct WithdrawAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub bumps: [u8; 1],
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    // Parses and validates the accounts from the slice provided by the entrypoint.
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 1. Unpack the accounts
        // We expect: [owner, vault, system_program (optional/implied)]
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 2. Perform Checks

        // Check 1: Ensure the owner is a signer.
        // We only allow withdrawal to the account that signed the transaction.
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Check 2: Verify the vault's owner.
        // The vault should be owned by the system program (since it holds lamports and is a PDA).
        // Wait, usually the vault is a PDA of THIS program.
        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Check 3: Business Logic / Data Validity
        // We ensure the vault is not empty before attempting to withdraw.
        if vault.lamports().eq(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check 4: PDA Validation
        // We re-derive the PDA address to ensure the 'vault' account passed is the correct one.
        // Seeds: "vault" + owner_pubkey
        let (vault_key, bump) = find_program_address(&[b"vault", owner.key().as_ref()], &crate::ID);
        if &vault_key != vault.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self {
            owner,
            vault,
            bumps: [bump],
        })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // Since Withdraw doesn't have instruction data (args), we only parse accounts.
        let accounts = WithdrawAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> Withdraw<'a> {
    // Unique discriminator for the Withdraw instruction (1).
    pub const DISCRIMINATOR: &'a u8 = &1;

    // Execution logic
    pub fn process(&mut self) -> ProgramResult {
        // 1. Prepare PDA Signers
        // The vault must sign to transfer funds out (since it's a PDA).
        // `find_program_address` returned the canonical bump, which we use here.
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key().as_ref()),
            Seed::from(&self.accounts.bumps),
        ];
        let signers = [Signer::from(&seeds)];

        // 2. Perform Transfer (CPI)
        // We invoke the System Program's Transfer instruction.
        // Signers are required because 'from' is a PDA.
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.owner,
            lamports: self.accounts.vault.lamports(),
        }
        .invoke_signed(&signers)?;

        Ok(())
    }
}
