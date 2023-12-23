use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

pub fn create_associated_token_account(
    payer: &Pubkey,
    owner: &Pubkey,
    token_mint: &Pubkey,
) -> (Pubkey, Instruction) {
    let ata = spl_associated_token_account::get_associated_token_address(owner, token_mint);
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        payer,
        owner,
        token_mint,
        &spl_token::ID,
    );
    (ata, ix)
}
