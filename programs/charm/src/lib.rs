use anchor_lang::prelude::*;
use spl_token_metadata::instruction::{create_master_edition, create_metadata_accounts};
use anchor_lang::solana_program::program::invoke;

declare_id!("BFXsCPze92SjvpLfh2axdkR4qUzjrambaDtodEJG7qa9");

#[program]
pub mod charm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, data: String) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let copy = data.clone();
        base_account.data = data;
        base_account.data_list.push(copy);
        Ok(())
    }

    pub fn update(ctx: Context<Update>, data: String) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        let copy = data.clone();
        base_account.data = data;
        base_account.data_list.push(copy);
        Ok(())
    }

    pub fn metadata(ctx: Context<Metadata>, data: String) -> ProgramResult{

        //First 20 chars as Name,
        let name = &data[0..20];
        //Next 4 as Symbol
        let symbol = &data[20..24];
        //Next 50 as URL
        let url = &data[24..74];
        msg!("{}, {}, {}", name.clone(), symbol.clone(), url.clone());

        let creators: Vec<spl_token_metadata::state::Creator> =
        vec![spl_token_metadata::state::Creator {
            address: *ctx.accounts.payer.key,
            verified: true,
            share: 100,
        }];
    msg!("Making metadata accounts vector...");
    let metadata_infos = vec![
        ctx.accounts.metadata_account.clone(),
        ctx.accounts.master_edition_account.clone(),
        ctx.accounts.metadata_program.clone(),
        ctx.accounts.mint.clone(),
        ctx.accounts.mint_authority.clone(),
        ctx.accounts.update_authority.clone(),
        ctx.accounts.payer.clone(),
        ctx.accounts.system_program.clone(),
        ctx.accounts.rent_program.clone(),
        ctx.accounts.token_program.clone(),
    ];
    msg!("Making metadata instruction");
    let instruction = create_metadata_accounts(
        *ctx.accounts.metadata_program.key,
        *ctx.accounts.metadata_account.key,
        *ctx.accounts.mint.key,
        *ctx.accounts.mint_authority.key,
        *ctx.accounts.payer.key,
        *ctx.accounts.update_authority.key,
         name.to_string(),
         symbol.to_string(),
         url.to_string(),
        Some(creators),
        //Default creator royality.. will be changed to client 
        20,
        //At the moment defaulting to update authority as signer as well... will be changed to client
        true,
        true,
    );
    msg!("Calling the metadata program to make metadata...");
    invoke(&instruction, metadata_infos.as_slice())?;

    msg!("Metadata created...");

    msg!("Creating master edition");
    let instruction_create_master_edition = create_master_edition(
        *ctx.accounts.metadata_program.key,
        *ctx.accounts.master_edition_account.key,
        *ctx.accounts.mint.key,
        *ctx.accounts.update_authority.key,
        *ctx.accounts.mint_authority.key,
        *ctx.accounts.metadata_account.key,
        *ctx.accounts.payer.key,
        //Default to 10 additonal prints... will update to client provided
        Some(10),
    );

    msg!("Calling the metadata program to make masteredition...");
    invoke(&instruction_create_master_edition, metadata_infos.as_slice(),)?;

    msg!("Metadata & editions created");
    

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 64 + 64)]
    pub base_account: Account<'info, BaseAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
}

#[derive(Accounts)]
pub struct Metadata<'info>{
    #[account(signer)]
    pub payer: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(signer)]
    pub mint_authority: AccountInfo<'info>,
    pub update_authority: AccountInfo<'info>,
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,
    #[account(mut)]
    pub master_edition_account: AccountInfo<'info>,
    #[account(executable)]
    pub metadata_program: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
    pub rent_program: AccountInfo<'info>
}

#[account]
pub struct BaseAccount {
    pub data: String,
    pub data_list: Vec<String>,
}
