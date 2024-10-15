pub mod merkle_tree;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata,
    },
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata as MetadataAccount},
    types::DataV2,
};

use crate::merkle_tree::*;

pub const ANCHOR_DISCRIMINATOR: usize = 8;

declare_id!("5FYgyRz5zunLbZ4BwHR7ALzjjtRbPqr2Xe5qvDXDq4x");

#[program]
pub mod test_nft {

    use anchor_lang::solana_program::keccak;
    use mpl_token_metadata::types::Collection;

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.nft_data.volume = 0;
        ctx.accounts.nft_data.total_supply = 20;

        Ok(())
    }

    pub fn verify(ctx: Context<Verify>, proof: Vec<[u8; 32]>) -> Result<()> {
        //init ctx variables
        let payer = &ctx.accounts.payer;
        let token_distributor = &mut ctx.accounts.merkle_data_account;
        //check that the owner is a Signer
        require!(ctx.accounts.payer.is_signer, MerkleError::Unauthorized);

        let leaf = keccak::hashv(&[&payer.key().to_bytes()]);

        require!(
            merkle_tree::verify(proof, token_distributor.root, leaf.0),
            MerkleError::InvalidProof
        );

        Ok(())
    }

    pub fn mint(
        ctx: Context<InitNFT>,
        name: String,
        symbol: String,
        uri: String,
        pubk: Pubkey,
    ) -> Result<()> {
        let volume: u16 = ctx.accounts.nft_data.volume;

        if volume > ctx.accounts.nft_data.total_supply {
            msg!("");
            return Ok(());
        }

        // create mint account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        );

        mint_to(cpi_context, 1)?;

        // create metadata account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        let data_v2 = DataV2 {
            name: name,
            symbol: symbol,
            uri: uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: Some(Collection {
                key: pubk.clone(),
                verified: false,
            }),
            uses: None,
        };

        create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

        //create master edition account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_master_edition_v3(cpi_context, Some(0))?;

        ctx.accounts.nft_data.volume += 1;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitNFT<'info> {
    /// CHECK: ok, we are passing in this account ourselves
    #[account(mut, signer)]
    pub signer: AccountInfo<'info>,

    pub collection_id: AccountInfo<'info>,

    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
    )]
    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    /// CHECK - address
    #[account(
        mut,
        address=MetadataAccount::find_pda(&mint.key()).0,
    )]
    pub metadata_account: AccountInfo<'info>,
    /// CHECK: address
    #[account(
        mut,
        address=MasterEdition::find_pda(&mint.key()).0,
    )]
    pub master_edition_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    #[account(
        mut,
        seeds = [
            b"nftData".as_ref(),
            signer.key().to_bytes().as_ref()
        ],
        bump,
    )]
    pub nft_data: Account<'info, NFTData>,

    #[account(
        mut,
        seeds = [
            b"MerkleTokenDistributor".as_ref(),
            signer.key().to_bytes().as_ref()
        ],
        bump,
    )]
    pub merkle_data_account: Account<'info, MerkleData>,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [
            b"MerkleTokenDistributor".as_ref(),
            signer.key().to_bytes().as_ref()
        ],
        bump,
        space = ANCHOR_DISCRIMINATOR + MerkleData::INIT_SPACE,
        payer = signer,
    )]
    pub merkle_data_account: Account<'info, MerkleData>,

    #[account(
        init,
        seeds = [
            b"nftData".as_ref(),
            signer.key().to_bytes().as_ref()
        ],
        bump,
        space = ANCHOR_DISCRIMINATOR + NFTData::INIT_SPACE,
        payer = signer,
    )]
    pub nft_data: Account<'info, NFTData>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct NFTData {
    volume: u16,
    total_supply: u16,
}

#[error_code]
pub enum MerkleError {
    #[msg("Invalid Merkle Proof")]
    InvalidProof,
    #[msg("Account is not authorized to execute this instruction")]
    Unauthorized,
    #[msg("Token account owner did not match intended owner")]
    OwnerMismatch,
    #[msg("Exceeded maximum mint amount.")]
    ExceededMaxMint,
}
