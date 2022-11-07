use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock;

use std::convert::TryInto;
pub use switchboard_v2::{
    history_buffer::AggregatorHistoryBuffer, AggregatorAccountData, SwitchboardDecimal,
    SWITCHBOARD_PROGRAM_ID, SWITCHBOARD_V2_DEVNET, SWITCHBOARD_V2_MAINNET,
};

declare_id!("GWLjdS5qvUUwTt8HHVTHmJ8F4ZmNYRvdNoiL8gBgc5H7");

#[program]

pub mod switchboard_feed_solana {
    use super::*;
    pub fn create_price_feed(ctx: Context<CreatePrizeFeedAccount>) -> Result<()> {
        let feed_vec_acc = &mut ctx.accounts.feed_vector_acc;
        feed_vec_acc.authority = ctx.accounts.authority.key();
        feed_vec_acc.data_spread = 0f64;

        Ok(())
    }

    pub fn append_feed_data(ctx: Context<ReadHistorybuffer>, period: i64) -> Result<()> {
        let mut period_insec = period;
        let feed_vector = &mut ctx.accounts.feed_vec_acc.feed_vector;
        let history_buffer = ctx.accounts.history_buffer.to_account_info();
        let history_buffer_acc = AggregatorHistoryBuffer::new(&history_buffer).unwrap();
        let cur_time = clock::Clock::get().unwrap().unix_timestamp;

        loop {
            let result = history_buffer_acc.lower_bound(cur_time - period_insec);
            match result {
                Some(data) => {
                    let sol_price: f64 = data.value.try_into().unwrap();
                    let timestamp = data.timestamp;
                    feed_vector.push((sol_price, timestamp));
                    period_insec = period_insec + period;
                }
                None => {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn calculate_data_spread(ctx: Context<CalculateDataSpread>, nod: u64) -> Result<()> {
        let feed_vec_acc = &mut ctx.accounts.feed_vector_acc;
        let feed_vector = &feed_vec_acc.feed_vector;
        let feed_arr = feed_vector.as_slice();

        let mut sum = 0f64;
        for i in feed_arr.iter() {
            sum = sum + i.0;
        }
        let mean = sum / (feed_arr.len() as f64);
        let mut sum_sq_devi = 0f64;
        for i in feed_arr.iter() {
            let dev_i2 = (i.0 - mean).powi(2);
            sum_sq_devi = sum_sq_devi + dev_i2;
        }
        let sd = (sum_sq_devi / (feed_arr.len() as f64)).sqrt();

        feed_vec_acc.data_spread = sd * (nod as f64).sqrt();
        msg!("{:?}", sd);
        return Ok(());
    }

    pub fn reset_vec_feed(ctx: Context<ResetFeedVec>) -> Result<()> {
        let feed_vec_acc = &mut ctx.accounts.feed_vec_acc;
        feed_vec_acc.feed_vector.clear();

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreatePrizeFeedAccount<'info> {
    #[account(init ,payer = authority,space = 8+1600+32)]
    pub feed_vector_acc: Account<'info, SolanaPriceFeed>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[derive(Debug)]
#[account]
pub struct SolanaPriceFeed {
    pub feed_vector: Vec<(f64, i64)>,
    pub data_spread: f64,
    pub authority: Pubkey,
}

#[derive(Accounts)]
pub struct ReadHistorybuffer<'info> {
    /// CHECK:safe account
    pub history_buffer: AccountInfo<'info>,
    #[account(mut,has_one = authority)]
    pub feed_vec_acc: Account<'info, SolanaPriceFeed>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CalculateDataSpread<'info> {
    #[account(mut,has_one = authority)]
    pub feed_vector_acc: Account<'info, SolanaPriceFeed>,
    pub authority: Signer<'info>,
}
#[derive(Accounts)]
pub struct ResetFeedVec<'info> {
    #[account(mut,has_one=authority)]
    pub feed_vec_acc: Account<'info, SolanaPriceFeed>,
    pub authority: Signer<'info>,
}
