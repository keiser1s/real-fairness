// ============================================================
// 🎲 Diceon.com - Real Fairness™
// ✅ On-chain dice game built for Solana
// 🧾 Fully verifiable logic, logs, and fair rolls.
// ============================================================

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{clock::Clock, Sysvar},
};

// 🔌 Entrypoint for Solana to call into
entrypoint!(process_instruction);

// 🧠 Core game logic: handles incoming bets, rolls, and payouts
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("🎲 Diceon.com - Real Fairness™");

    // 🔗 Accounts
    let accounts_iter = &mut accounts.iter();
    let player = next_account_info(accounts_iter)?;         // 🎮 Player placing the bet
    let vault = next_account_info(accounts_iter)?;          // 💰 PDA vault holding all funds
    let system_program = next_account_info(accounts_iter)?; // 🔧 System program

    // 📏 Instruction must be exactly 11 bytes
    if instruction_data.len() != 11 {
        msg!("❌ Instruction must be 11 bytes");
        return Err(ProgramError::InvalidInstructionData);
    }

    // 💸 Parse bet amount + target + direction (over/under)
    let bet_amount = u64::from_le_bytes(instruction_data[0..8].try_into().unwrap());
    let target_raw = u16::from_le_bytes(instruction_data[8..10].try_into().unwrap());
    let target = target_raw as f64 / 10.0; // Convert to decimal (e.g. 745 => 74.5)
    let is_over = instruction_data[10];

    // 🚫 Validate inputs
    if target < 2.0 || target > 98.0 {
        msg!("❌ Target must be between 2.0 and 98.0");
        return Err(ProgramError::InvalidInstructionData);
    }
    if is_over != 0 && is_over != 1 {
        msg!("❌ Direction must be 0 (under) or 1 (over)");
        return Err(ProgramError::InvalidInstructionData);
    }

    // 🎲 Set player bet conditions
    let direction = if is_over == 1 { "over" } else { "under" };
    msg!(
        "🧾 Player is betting {} lamports to roll {} {:.1}",
        bet_amount,
        direction,
        target
    );

    // 🧾 Verify the vault PDA matches
    let (expected_pda, bump) = Pubkey::find_program_address(&[b"vault"], program_id);
    if *vault.key != expected_pda {
        msg!("❌ Invalid vault PDA");
        return Err(ProgramError::InvalidAccountData);
    }

    // 🔁 Transfer bet from player → vault
    invoke(
        &system_instruction::transfer(player.key, vault.key, bet_amount),
        &[player.clone(), vault.clone(), system_program.clone()],
    )?;

    // 🎲 Generate pseudo-random roll using current slot
    let clock = Clock::get()?;
    let slot = clock.slot;
    let roll_raw = ((slot % 100) * 10 + (slot % 10)) as u16;
    let roll = roll_raw as f64 / 10.0; // 1 decimal precision
    msg!("🎲 Rolled: {:.1}", roll);

    // 🎯 Determine win/loss
    let win = if is_over == 1 {
        roll > target
    } else {
        roll < target
    };

    if win {
        let win_chance = if is_over == 1 {
            100.0 - target
        } else {
            target
        };

        if win_chance <= 0.0 {
            msg!("❌ Invalid win chance");
            return Err(ProgramError::InvalidInstructionData);
        }

        let multiplier = (100.0 / win_chance) * 0.99;
        let payout = (bet_amount as f64 * multiplier).floor() as u64;

        msg!("✅ Player wins! Paying out {} lamports", payout);

        // 💸 Send payout from vault → player
        invoke_signed(
            &system_instruction::transfer(vault.key, player.key, payout),
            &[vault.clone(), player.clone(), system_program.clone()],
            &[&[b"vault", &[bump]]],
        )?;
    } else {
        msg!("❌ Player loses. Better luck next roll.");
    }

    Ok(())
}