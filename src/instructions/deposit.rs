use core::mem::size_of;
use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};

// In Pinocchio, we don't use macros like `#[derive(Accounts)]` from Anchor.
// Instead, we define a struct to hold the accounts and implement `TryFrom` to parse and validate them manually.
// This gives us full control over the number of checks and optimizations (Zero-Copy).
pub struct DepositAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    // This method parses the array of accounts passed by the runtime.
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // 1. Destructure the accounts array.
        // We expect specific accounts in a specific order.
        // 'owner': The signer paying for the transaction or deposit.
        // 'vault': The PDA account where funds will be deposited.
        // '_': Use `_` to ignore extra accounts if any (like system program).
        // If the number of accounts doesn't match, we return an error.
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 2. Perform Validation Checks
        // Unlike Anchor, which generates these checks for you, here we write them explicitly.

        // Check 1: Ensure the owner signed the transaction.
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Check 2: Verification of the Vault's owner.
        // Pinocchio system might need to own the vault, or it should be a PDA of this program.
        // Here it checks if it's owned by `pinocchio_system::ID`. (Adjust based on actual logic intent).
        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Check 3: Ensure the vault is empty (lamports == 0) for this specific 'Deposit' logic context
        // (This seems to imply this deposit might be initializing or expecting an empty state,
        // or just a specific business rule).
        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        // Check 4: PDA Validation.
        // We verify that the 'vault' account is indeed the correct PDA derived from "vault" + owner public key.
        // This protects against fake vault accounts being passed.
        let (vault_key, _) = find_program_address(&[b"vault", owner.key().as_ref()], &crate::ID);
        if vault.key().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Return the validated struct
        Ok(Self { owner, vault })
    }
}

// Struct to hold the instruction data (variables passed to the function).
// In Anchor, this would be the arguments to the function handler.
pub struct DepositInstructionData {
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData {
    type Error = ProgramError;

    // deserializes the raw byte array into the struct.
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // 1. Check data length.
        // We expect exactly 8 bytes for a u64 amount.
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // 2. Parse the data.
        // We convert the first 8 bytes into a u64 accumulator.
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // 3. Logic Checks on Data
        // Ensure the amount is greater than 0.
        if amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self { amount })
    }
}

// The main context struct for the Deposit instruction.
// Creates a unified view of both Accounts and Data.
pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instruction_data: DepositInstructionData,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        // Parse accounts and data separately using the implementations above.
        let accounts = DepositAccounts::try_from(accounts)?;
        let instruction_data = DepositInstructionData::try_from(data)?;

        Ok(Self {
            accounts,
            instruction_data,
        })
    }
}

impl<'a> Deposit<'a> {
    // Discriminator is used in `lib.rs` to route the instruction.
    // In Anchor, this is an 8-byte hash, but here we can just use a simple `u8` (0 for Deposit).
    pub const DISCRIMINATOR: &'a u8 = &0;

    // The business logic of the instruction.
    pub fn process(&mut self) -> ProgramResult {
        // Execute a Cross-Program Invocation (CPI) to transfer lamports.
        // We construct a `Transfer` instruction (likely a helper struct/method defined elsewhere or in a library)
        // and invoke it.
        // This moves `amount` lamports from `owner` to `vault`.
        pinocchio_system::instructions::Transfer {
            from: self.accounts.owner,
            to: self.accounts.vault,
            lamports: self.instruction_data.amount,
        }
        .invoke()?;

        Ok(())
    }
}
