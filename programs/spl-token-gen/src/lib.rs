#![allow(unused)]

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction, system_program};

use std::mem::size_of;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod spl_token_gen {
    use super::*;

    pub fn init(
        ctx: Context<Init>,
        key: Vec<u8>,
        storage_reference_bump_seed: u8,
    ) -> ProgramResult {
        let initial_storage = Pubkey::find_program_address(
            &[b"init", ctx.accounts.storage_reference.key().as_ref()],
            ctx.program_id,
        );
        let (initial_storage, initial_storage_bump_seed) = initial_storage;
        assert_eq!(&initial_storage, ctx.accounts.initial_storage.key);

        {
            let from = ctx.accounts.payer.key;
            let to = ctx.accounts.initial_storage.key;
            let lamports = 100000;
            let space = HEADER_BYTES;
            let owner = ctx.program_id;

            let ix = system_instruction::create_account(from, to, lamports, space, owner);

            invoke_signed(
                &ix,
                &[
                    ctx.accounts.payer.clone(),
                    ctx.accounts.initial_storage.clone(),
                    ctx.accounts.system_program.clone(),
                ],
                &[&[
                    b"init",
                    ctx.accounts.storage_reference.key().as_ref(),
                    &[initial_storage_bump_seed],
                ]],
            )?;
        }

        ctx.accounts.storage_reference.storage = initial_storage;

        Ok(())
    }

    pub fn set(ctx: Context<Set>, value: Vec<u8>) -> ProgramResult {
        set_or_clear(ctx, Some(value))
    }

    pub fn clear(ctx: Context<Set>) -> ProgramResult {
        set_or_clear(ctx, None)
    }
}

pub fn set_or_clear(ctx: Context<Set>, value: Option<Vec<u8>>) -> ProgramResult {
    let next_storage_seeds = &[b"next", ctx.accounts.storage.key.as_ref()];
    let next_storage = Pubkey::find_program_address(next_storage_seeds, ctx.program_id);
    let (next_storage, next_storage_bump_seed) = next_storage;
    assert_eq!(&next_storage, ctx.accounts.next_storage.key);

    {
        let from = ctx.accounts.payer.key;
        let to = ctx.accounts.next_storage.key;
        let lamports = 10000;
        let space = HEADER_BYTES + value.as_ref().map(Vec::len).unwrap_or_default() as u64;
        let owner = ctx.program_id;

        invoke_signed(
            &system_instruction::create_account(from, to, lamports, space, owner),
            &[
                ctx.accounts.payer.clone(),
                ctx.accounts.next_storage.clone(),
                ctx.accounts.system_program.clone(),
            ],
            &[&[
                b"next",
                ctx.accounts.storage.key.as_ref(),
                &[next_storage_bump_seed],
            ]],
        )?;
    }

    {
        let mut data = ctx.accounts.next_storage.data.borrow_mut();

        if let Some(value) = value {
            data[1..].copy_from_slice(&value);
            data[0] = HAVE_VALUE;
        } else {
            assert_eq!(data[0], 0);
        }
    }

    ctx.accounts.storage_reference.storage = next_storage;

    Ok(())
}

const HEADER_BYTES: u64 = 1;
const HAVE_VALUE: u8 = 0xA1;

#[derive(Accounts)]
#[instruction(key: Vec<u8>, storage_reference_bump_seed: u8)]
pub struct Init<'info> {
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    #[account(
        init, payer = payer, space = 8 + size_of::<StorageReference>(),
        seeds = [b"key", payer.key.as_ref(), key.as_ref()],
        bump = storage_reference_bump_seed
    )]
    pub storage_reference: ProgramAccount<'info, StorageReference>,
    #[account(
        mut,
        constraint = initial_storage.owner == &system_program::ID,
        constraint = initial_storage.data.borrow().is_empty(),
    )]
    pub initial_storage: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Set<'info> {
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    #[account(mut, has_one = storage)]
    pub storage_reference: ProgramAccount<'info, StorageReference>,
    #[account(mut, owner = *program_id)]
    pub storage: AccountInfo<'info>,
    #[account(
        mut,
        constraint = next_storage.owner == &system_program::ID,
        constraint = next_storage.data.borrow().is_empty(),
    )]
    pub next_storage: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: AccountInfo<'info>,
}

#[account]
#[derive(Default)]
pub struct StorageReference {
    pub storage: Pubkey,
}
