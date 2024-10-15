use anchor_lang::prelude::*;

pub const ANCHOR_DISCRIMINATOR: usize = 8;

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct MerkleData {
    // 256-bit Merkle root
    pub root: [u8; 32],
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [
            b"MerkleTokenDistributor".as_ref(),
            payer.key().to_bytes().as_ref()
        ],
        bump,
        space = ANCHOR_DISCRIMINATOR + MerkleData::INIT_SPACE,
        payer = payer,
    )]
    pub merkle_data_account: Account<'info, MerkleData>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Verify<'info> {
    #[account(
        mut,
        seeds = [
            b"MerkleTokenDistributor", 
            payer.key().to_bytes().as_ref(),
        ],
        bump,
    )]
    pub merkle_data_account: Account<'info, MerkleData>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
