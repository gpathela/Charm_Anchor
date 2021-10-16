use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, Transfer};
use spl_associated_token_account::{create_associated_token_account};
use spl_token_metadata::{
    instruction::{
        create_master_edition, create_metadata_accounts,
        mint_new_edition_from_master_edition_via_token, puff_metadata_account,
    },
    state::Metadata,
};

use anchor_lang::solana_program::{
    borsh::try_from_slice_unchecked,
    program::{invoke, invoke_signed},
};

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

    pub fn set_authority(ctx: Context<SetAuthority>) -> ProgramResult {
        //This method is only required in Dev.
        //Only purpose of this method is to give ownership of a mint to PDA
        //So that it can mint tokens later
        let ix = spl_token::instruction::set_authority(
            &ctx.accounts.token_program.key,
            &ctx.accounts.mint.key,
            Some(&ctx.accounts.pda.key),
            spl_token::instruction::AuthorityType::MintTokens,
            &ctx.accounts.signer.key,
            &[&ctx.accounts.signer.key],
        )?;
        msg!("Calling program to transfer min ownership");
        invoke(
            &ix,
            &[
                ctx.accounts.mint.clone(),
                ctx.accounts.signer.clone(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;

        Ok(())
    }

    pub fn create_associated_account(ctx: Context<CreateAssociated>) -> Result<()>{
        
        msg!("Creating Associated account");
        let create_ix = create_associated_token_account(
            &ctx.accounts.signer.key,
            &ctx.accounts.signer.key,
            &ctx.accounts.mint.key,
        );
        let associated_required_accounts = vec![
            ctx.accounts.signer.clone(),
            ctx.accounts.user_account.clone(),
            ctx.accounts.signer.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.rent_program.to_account_info(),
            ctx.accounts.associated_program.clone(),
        ];

        msg!("Invoking instruction to create account 1111 trying");
        invoke(&create_ix, &associated_required_accounts)?;

        Ok(())

    }

    pub fn faucet(ctx: Context<Faucet>, bump: u8) -> Result<()> {

        
        msg!("Making instruction for faucet");
        let ix = spl_token::instruction::mint_to(
            &ctx.accounts.token_program.key,
            &ctx.accounts.mint.key,
            &ctx.accounts.user_account.key,
            &ctx.accounts.pda.key,
            &[&ctx.accounts.pda.key],
            100 * 100_00_00,
        )?;
        msg!("Invoking instruction for faucet");
        invoke_signed(
            &ix,
            &[
                ctx.accounts.mint.clone(),
                ctx.accounts.user_account.clone(),
                ctx.accounts.pda.clone(),
                ctx.accounts.token_program.to_account_info(),
            ],
            &[&[&b"charmpda"[..], &[bump]]],
        )?;

        Ok(())
    }

    pub fn puff_metadata(ctx: Context<PuffMetadata>) -> ProgramResult {
        msg!("Puff metadata ");

        let ix = puff_metadata_account(
            *ctx.accounts.metadata_program.key,
            *ctx.accounts.metadata_account.key,
        );

        invoke(
            &ix,
            &[
                ctx.accounts.mint.clone(),
                ctx.accounts.signer.clone(),
                ctx.accounts.metadata_account.clone(),
                ctx.accounts.metadata_program.to_account_info(),
            ],
        )?;
        Ok(())
    }

    pub fn proxy_transfer(ctx: Context<ProxyTransfer>, amount: u64) -> ProgramResult {
        msg!("Transffering tokens");
        token::transfer(ctx.accounts.into(), amount)?;
        Ok(())
    }

    pub fn buy(ctx: Context<Buy>, edition: u64, _pda_nonce: u8) -> ProgramResult {
        msg!("Making buy accounts vector...");
        let metadata_infos = vec![
            ctx.accounts.metadata_program.clone(),
            ctx.accounts.new_metadata_account.clone(),
            ctx.accounts.new_edition_account.clone(),
            ctx.accounts.master_edition_account.clone(),
            ctx.accounts.new_mint_account.clone(),
            ctx.accounts.new_mint_authority.clone(),
            ctx.accounts.payer.to_account_info().clone(),
            ctx.accounts.token_account_owner.clone(),
            ctx.accounts.token_account.clone(),
            ctx.accounts.new_metadata_update_authority.clone(),
            ctx.accounts.metadata.clone(),
            ctx.accounts.metadata_mint.clone(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent_program.to_account_info(),
            ctx.accounts.edition_pda.clone(),
            ctx.accounts.token_program.to_account_info(),
        ];

        msg!("Creating edition from master edition");
        let ix = mint_new_edition_from_master_edition_via_token(
            *ctx.accounts.metadata_program.key,
            *ctx.accounts.new_metadata_account.key,
            *ctx.accounts.new_edition_account.key,
            *ctx.accounts.master_edition_account.key,
            *ctx.accounts.new_mint_account.key,
            *ctx.accounts.new_mint_authority.key,
            *ctx.accounts.payer.key,
            *ctx.accounts.token_account_owner.key,
            *ctx.accounts.token_account.key,
            *ctx.accounts.new_metadata_update_authority.key,
            *ctx.accounts.metadata.key,
            *ctx.accounts.metadata_mint.key,
            edition,
        );

        msg!("Calling the metadata program to make edition...");
        invoke(&ix, metadata_infos.as_slice())?;
        /* invoke_signed(
            &ix,
            metadata_infos.as_slice(),
            &[&[b"charmpda", &[pda_nonce]]],
        )?; */

        Ok(())
    }

    pub fn metadata(ctx: Context<CreateMetadata>, data: String) -> ProgramResult {
        //First 20 chars as Name,
        let name = &data[0..20];
        //Next 4 as Symbol
        let symbol = &data[20..24];
        //Next 65 as ARWEAVE URL
        let url = &data[24..140];
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
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
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

        //let data = Metadata::unpack(&ctx.accounts.metadata_account.data.borrow());
        let metadata_data: Metadata =
            try_from_slice_unchecked(&ctx.accounts.metadata_account.data.borrow())?;

        msg!("Hopefully got data");
        msg!(&metadata_data.data.uri);
        Ok(())
    }

    //Adding a seprate call for minting master edition
    pub fn edition(ctx: Context<Edition>) -> ProgramResult {
        msg!("Making edition accounts vector...");
        let metadata_infos = vec![
            ctx.accounts.metadata_account.clone(),
            ctx.accounts.master_edition_account.clone(),
            ctx.accounts.metadata_program.clone(),
            ctx.accounts.mint.clone(),
            ctx.accounts.mint_authority.clone(),
            ctx.accounts.update_authority.clone(),
            ctx.accounts.payer.clone(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ];

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
            Some(1),
        );

        msg!("Calling the metadata program to make masteredition...");
        invoke(
            &instruction_create_master_edition,
            metadata_infos.as_slice(),
        )?;
        msg!("Metadata & editions created");
        msg!("Transffering master edition token ownership to charmpda");

        Ok(())
    }

    pub fn change_ownership(ctx: Context<ChangeOwnership>) -> ProgramResult {
        msg!("Creating instruction to change master edition ownership to PDA");
        let owner_change_ix = spl_token::instruction::set_authority(
            &ctx.accounts.token_program.key,
            &ctx.accounts.master_edition_account.key,
            Some(&ctx.accounts.pda.key),
            spl_token::instruction::AuthorityType::AccountOwner,
            &ctx.accounts.signer.key,
            &[&ctx.accounts.signer.key],
        )?;
        msg!("Calling the token program to transfer aster edition token account ownership...");
        invoke(
            &owner_change_ix,
            &[
                ctx.accounts.master_edition_account.clone(),
                ctx.accounts.signer.clone(),
                ctx.accounts.token_program.to_account_info().clone(),
            ],
        )?;

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
pub struct SetAuthority<'info> {
    #[account(signer)]
    pub signer: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    pub pda: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Faucet<'info> {
    #[account(signer)]
    pub signer: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: AccountInfo<'info>,
    pub pda: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
    //pub system_program: Program<'info, System>,
    //pub rent_program: Sysvar<'info, Rent>,
    //#[account(executable)]
    //pub associated_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct CreateAssociated<'info> {
    #[account(signer)]
    pub signer: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub user_account: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent_program: Sysvar<'info, Rent>,
    #[account(executable)]
    pub associated_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct PuffMetadata<'info> {
    #[account(signer)]
    pub signer: AccountInfo<'info>,
    #[account(mut)]
    pub metadata_account: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    #[account(executable)]
    pub metadata_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(executable)]
    pub metadata_program: AccountInfo<'info>,
    #[account(mut)]
    pub new_metadata_account: AccountInfo<'info>,
    #[account(mut)]
    pub new_edition_account: AccountInfo<'info>,
    #[account(mut)]
    pub master_edition_account: AccountInfo<'info>,
    #[account(mut)]
    pub new_mint_account: AccountInfo<'info>,
    pub new_mint_authority: AccountInfo<'info>,
    pub token_account_owner: AccountInfo<'info>,
    pub token_account: AccountInfo<'info>,
    pub new_metadata_update_authority: AccountInfo<'info>,
    pub metadata: AccountInfo<'info>,
    pub metadata_mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub rent_program: Sysvar<'info, Rent>,
    #[account(mut)]
    pub edition_pda: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ChangeOwnership<'info> {
    #[account(signer)]
    pub signer: AccountInfo<'info>,
    #[account(mut)]
    pub master_edition_account: AccountInfo<'info>,
    pub pda: AccountInfo<'info>,
    #[account(executable)]
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CreateMetadata<'info> {
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent_program: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Edition<'info> {
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent_program: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ProxyTransfer<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub from: AccountInfo<'info>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[account]
pub struct BaseAccount {
    pub data: String,
    pub data_list: Vec<String>,
}

impl<'a, 'b, 'c, 'info> From<&mut ProxyTransfer<'info>>
    for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>>
{
    fn from(accounts: &mut ProxyTransfer<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: accounts.from.clone(),
            to: accounts.to.clone(),
            authority: accounts.authority.clone(),
        };
        let cpi_program = accounts.token_program.clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Insufficient Balance")]
    InsufficientBalance,
}
