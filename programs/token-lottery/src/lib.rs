use anchor_lang::{accounts::signer, prelude::*, system_program};
use anchor_spl::{associated_token::AssociatedToken, 
    token_interface::{Mint, TokenAccount, TokenInterface, mint_to, MintTo},
    
};
use switchboard_on_demand::accounts::RandomnessAccountData;

use anchor_spl::metadata::{
    Metadata,
    MetadataAccount,
    CreateMetadataAccountsV3,
    CreateMasterEditionV3,
    SignMetadata,
    SetAndVerifySizedCollectionItem,
    create_master_edition_v3,
    create_metadata_accounts_v3,
    sign_metadata,
    set_and_verify_sized_collection_item,
    mpl_token_metadata::types::{
            CollectionDetails,
            Creator, 
            DataV2,
        },
};

declare_id!("FL4Bcmn5gqstfAub6D4A6P5UgrbHJ7StxHgiR36LcGDx");
#[constant]
pub const NAME: &str = "Token Lottery Ticket #";
#[constant]
pub const SYMBOL: &str = "TLT";
#[constant]
pub const URI: &str = "https://bronze-voluntary-shrimp-569.mypinata.cloud/ipfs/bafkreiabta4eqjfxxj5hr3usdy53i6yryve7punrwsxscsbotr2uggggbq"; 

#[program]
pub mod token_lottery {
    use super::*;

    pub fn initialize_config(ctx: Context<Initialize>, start: u64, end: u64, price: u64) -> Result<()> {
        ctx.accounts.token_lottery.bump = ctx.bumps.token_lottery;
        ctx.accounts.token_lottery.lottery_start = start;
        ctx.accounts.token_lottery.lottery_end = end;
        ctx.accounts.token_lottery.price = price;
        ctx.accounts.token_lottery.authority = ctx.accounts.payer.key();
        ctx.accounts.token_lottery.winner = 0;
        ctx.accounts.token_lottery.winner_chosen = false;
        ctx.accounts.token_lottery.lottery_pot_amount = 0;
        ctx.accounts.token_lottery.total_tickets = 0;
        ctx.accounts.token_lottery.randomness_account = Pubkey::default(); // Set to a   
        Ok(())
    }
    pub fn initialize_lottery(ctx: Context<InitializeLottery>) -> Result<()> {
        
        let signer_seeds: &[&[&[u8]]] = &[&[
            b"collection_mint".as_ref(),
            &[ctx.bumps.collection_mint],
        ]];

        msg!("Creating mint accounts");
        mint_to(
            CpiContext::new_with_signer(
                 ctx.accounts.token_program.to_account_info(),
                 MintTo{
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    to: ctx.accounts.collection_token_account.to_account_info(),
                    authority: ctx.accounts.collection_mint.to_account_info(),
                 },
                 signer_seeds,
            ),
            1, // Mint 1 token
        )?;

        msg!("Creating Metadata account");
        create_metadata_accounts_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds
            ),
            
            DataV2 {
                name:NAME.to_string(),
                symbol:SYMBOL.to_string(),
                uri:URI.to_string(),
                seller_fee_basis_points: 0,
                creators: Some(vec![Creator{
                    address: ctx.accounts.collection_mint.key(),
                    verified: false,
                    share: 100,
                }]), 
                collection: None,
                uses: None,
            }, 
            true, 
            true, 
            Some(CollectionDetails::V1 {
                size: 0, // Set size to 0 for no collection

            }), 
        )?;      

        msg!("Creating Master Edition account");
        create_master_edition_v3(
            CpiContext::new_with_signer(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMasterEditionV3 {
                    payer: ctx.accounts.payer.to_account_info(),
                    mint: ctx.accounts.collection_mint.to_account_info(),
                    edition: ctx.accounts.master_edition.to_account_info(),
                    update_authority: ctx.accounts.collection_mint.to_account_info(),
                    mint_authority: ctx.accounts.collection_mint.to_account_info(),
                    metadata: ctx.accounts.metadata.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                &signer_seeds,
            ),
            Some(0), 
        )?;

        msg!("Verifying collection item");
        sign_metadata(CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            SignMetadata {
                metadata: ctx.accounts.metadata.to_account_info(),
                creator: ctx.accounts.collection_mint.to_account_info(),
            },
            signer_seeds,
        ))?;
        Ok(())
    }    
    pub fn buy_ticket(ctx: Context<BuyTicket>) -> Result<()> {
    let clock = Clock::get()?;
    let ticket_name = NAME.to_owned() + ctx.accounts.token_lottery.ticket_num.to_string().as_str();

    msg!("Buying ticket: {}", ticket_name);

    if clock.slot < ctx.accounts.token_lottery.lottery_start || 
        clock.slot > ctx.accounts.token_lottery.lottery_end as u64 {
        return Err(ErrorCode::LotteryNotOpen.into());
    }   

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.payer.to_account_info(),
                to: ctx.accounts.token_lottery.to_account_info(),
            },
        ),
        ctx.accounts.token_lottery.price,
    )?;
    
    ctx.accounts.token_lottery.lottery_pot_amount += ctx.accounts.token_lottery.price;

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"collection_mint".as_ref(),
        &[ctx.bumps.collection_mint],
    ]];

    //Mint ticket
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.ticket_mint.to_account_info(),
                to: ctx.accounts.destination.to_account_info(),
                authority: ctx.accounts.collection_mint.to_account_info(),
            },
            &signer_seeds,
        ),
        1,
    )?;

    create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                payer: ctx.accounts.payer.to_account_info(),    
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds
        ),
        DataV2 { name: ticket_name, symbol: SYMBOL.to_string(), uri: URI.to_string(), seller_fee_basis_points: 0, creators: None, collection: None, uses: None },
        true,
        true,
        None
    )?;

    create_master_edition_v3(
        CpiContext::new_with_signer(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                payer: ctx.accounts.payer.to_account_info(),
                mint: ctx.accounts.ticket_mint.to_account_info(),
                edition: ctx.accounts.ticket_master_edition.to_account_info(),
                mint_authority: ctx.accounts.collection_mint.to_account_info(),
                update_authority: ctx.accounts.collection_mint.to_account_info(),
                metadata: ctx.accounts.ticket_metadata.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent : ctx.accounts.rent.to_account_info(),
            },
            &signer_seeds,
        ),
        Some(0),
    )?;
    set_and_verify_sized_collection_item(
    CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        SetAndVerifySizedCollectionItem {
            metadata: ctx.accounts.ticket_metadata.to_account_info(),
            collection_authority: ctx.accounts.collection_mint.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            update_authority: ctx.accounts.collection_mint.to_account_info(),
            collection_mint: ctx.accounts.collection_mint.to_account_info(),
            collection_metadata: ctx.accounts.collection_metadata.to_account_info(),
            collection_master_edition: ctx
                .accounts
                .collection_master_edition
                .to_account_info(),
            },
            &signer_seeds,
        ),
        None,
    )?;

    ctx.accounts.token_lottery.ticket_num += 1;
    msg!("Ticket number: {}", ctx.accounts.token_lottery.ticket_num);

    Ok(())
}
    pub fn commit_a_winner(ctx: Context<CommitWinner>) -> Result<()> {
        let clock = Clock::get()?;
        let token_lottery = &mut ctx.accounts.token_lottery;
        if ctx.accounts.payer.key() != token_lottery.authority {
            return Err(ErrorCode::NotAuthorized.into());
        }

        let randomness_data = RandomnessAccountData::parse(ctx.accounts.randomness_account_data.data.borrow()).unwrap();

        if randomness_data.seed_slot != clock.slot - 1 {
            return Err(ErrorCode::RandomnessAlreadyRevealed.into());
        }

        token_lottery.randomness_account = ctx.accounts.randomness_account_data.key();

        Ok(())
    }
    pub fn choose_a_winner(ctx: Context<ChooseWinner>) -> Result<()> {
        let clock = Clock::get()?;
        let token_lottery = &mut ctx.accounts.token_lottery;

        if ctx.accounts.randomness_account_data.key() != token_lottery.randomness_account {
            return Err(ErrorCode::IncorrectRandomnessAccount.into());
        }
        if ctx.accounts.payer.key() != token_lottery.authority {
            return Err(ErrorCode::NotAuthorized.into());
        }   
        if clock.slot < token_lottery.lottery_end {
            msg!("Current slot: {}", clock.slot);
            msg!("End slot: {}", token_lottery.lottery_end);
            return Err(ErrorCode::LotteryNotCompleted.into());
        }
        require!(token_lottery.winner_chosen == false, ErrorCode::WinnerChosen);

        let randomness_data = 
            RandomnessAccountData::parse(ctx.accounts.randomness_account_data.data.borrow()).unwrap();
        let revealed_random_value = randomness_data.get_value(&clock)
            .map_err(|_| ErrorCode::RandomnessNotResolved)?;

        msg!("Randomness result: {}", revealed_random_value[0]);
        msg!("Ticket num: {}", token_lottery.ticket_num);

        let randomness_result = 
            revealed_random_value[0] as u64 % token_lottery.ticket_num;

        msg!("Winner: {}", randomness_result);

        token_lottery.winner = randomness_result;
        token_lottery.winner_chosen = true;

        Ok(())
    }
    pub fn claim_prize(ctx: Context<ClaimPrize>) -> Result<()> {
        // Check if winner has been chosen
        msg!("Winner chosen: {}", ctx.accounts.token_lottery.winner_chosen);
        require!(ctx.accounts.token_lottery.winner_chosen, ErrorCode::WinnerNotChosen);
        
        // Check if token is a part of the collection
        require!(ctx.accounts.metadata.collection.as_ref().unwrap().verified, ErrorCode::NotVerifiedTicket);
        require!(ctx.accounts.metadata.collection.as_ref().unwrap().key == ctx.accounts.collection_mint.key(), ErrorCode::IncorrectTicket);

        let ticket_name = NAME.to_owned() + &ctx.accounts.token_lottery.winner.to_string();
        let metadata_name = ctx.accounts.metadata.name.replace("\u{0}", "");


        msg!("Ticket name: {}", ticket_name);
        msg!("Metdata name: {}", metadata_name);

        // Check if the winner has the winning ticket
        require!(metadata_name == ticket_name, ErrorCode::IncorrectTicket);
        require!(ctx.accounts.destination.amount > 0, ErrorCode::IncorrectTicket);

        **ctx.accounts.token_lottery.to_account_info().try_borrow_mut_lamports()? -= ctx.accounts.token_lottery.lottery_pot_amount;
        **ctx.accounts.payer.try_borrow_mut_lamports()? += ctx.accounts.token_lottery.lottery_pot_amount;

        ctx.accounts.token_lottery.lottery_pot_amount = 0;

        Ok(())
    }
}



 

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + TokenLottery::INIT_SPACE, // Adjust space as needed for your data
        seeds = [b"token_lottery".as_ref()],
        bump
    )]
    pub token_lottery: Account<'info, TokenLottery>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CommitWinner<'info> {    
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: This account is checked by the Switchboard smart contract
    pub randomness_account_data: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeLottery<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        seeds = [b"collection_mint".as_ref()],
        bump
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = collection_mint,
        token::authority = collection_token_account,
        seeds = [b"collection_token_account".as_ref()],
        bump
    )]
    pub collection_token_account: InterfaceAccount<'info, TokenAccount>,

    
    #[account(
        mut,
        seeds = [
                b"metadata",
                token_metadata_program.key().as_ref(), 
                collection_mint.key().as_ref(),
                ],
        bump,
        seeds::program = token_metadata_program.key(),
        
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub metadata: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"metadata",
                token_metadata_program.key().as_ref(), 
                collection_mint.key().as_ref(),
                b"edition"
                ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub master_edition: UncheckedAccount<'info>,    

    pub token_metadata_program: Program<'info, Metadata>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}  

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        init,
        payer = payer,
        seeds = [token_lottery.ticket_num.to_le_bytes().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = collection_mint,
        mint::freeze_authority = collection_mint,
        mint::token_program = token_program,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = payer,
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
                b"metadata",
                token_metadata_program.key().as_ref(), 
                ticket_mint.key().as_ref(),
                ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub ticket_metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata",
                token_metadata_program.key().as_ref(), 
                ticket_mint.key().as_ref(),
                b"edition"
                ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub ticket_master_edition: UncheckedAccount<'info>,    

    #[account(
        mut,
        seeds = [
                b"metadata",
                token_metadata_program.key().as_ref(), 
                collection_mint.key().as_ref(),
                ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub collection_metadata: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"metadata",
                token_metadata_program.key().as_ref(), 
                collection_mint.key().as_ref(),
                b"edition"
                ],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    /// CHECK: This account is checked by the metadata smart contract
    pub collection_master_edition: UncheckedAccount<'info>, 

    
    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub token_metadata_program: Program<'info, Metadata>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct ChooseWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    /// CHECK: The account's data is validated manually within the handler.
    pub randomness_account_data: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct TokenLottery {
    bump: u8,
    pub winner: u64,
    pub winner_chosen: bool,
    pub lottery_start: u64,
    pub lottery_end: u64,
    pub lottery_pot_amount: u64,
    pub total_tickets: u64,
    pub ticket_num: u64,
    pub price: u64,
    pub authority: Pubkey,
    pub randomness_account: Pubkey,
}

#[derive(Accounts)]
pub struct ClaimPrize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_lottery".as_ref()],
        bump = token_lottery.bump,
    )]
    pub token_lottery: Account<'info, TokenLottery>,

    #[account(
        mut,
        seeds = [b"collection_mint".as_ref()],
        bump,
    )]
    pub collection_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [token_lottery.winner.to_le_bytes().as_ref()],
        bump,
    )]
    pub ticket_mint: InterfaceAccount<'info, Mint>,

    #[account(
        seeds = [b"metadata", token_metadata_program.key().as_ref(), ticket_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata: Account<'info, MetadataAccount>,

    #[account(
        associated_token::mint = ticket_mint,
        associated_token::authority = payer,
        associated_token::token_program = token_program,
    )]
    pub destination: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), collection_mint.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub token_metadata_program: Program<'info, Metadata>,
}
#[error_code]
pub enum ErrorCode {
    #[msg("Incorrect randomness account")]
    IncorrectRandomnessAccount,
    #[msg("Lottery not completed")]
    LotteryNotCompleted,
    #[msg("Lottery is not open")]
    LotteryNotOpen,
    #[msg("Not authorized")]
    NotAuthorized,
    #[msg("Randomness already revealed")]
    RandomnessAlreadyRevealed,
    #[msg("Randomness not resolved")]
    RandomnessNotResolved,
    #[msg("Winner not chosen")]
    WinnerNotChosen,
    #[msg("Winner already chosen")]
    WinnerChosen,
    #[msg("Ticket is not verified")]
    NotVerifiedTicket,
    #[msg("Incorrect ticket")]
    IncorrectTicket,
}