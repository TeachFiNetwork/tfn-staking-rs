use crate::common::{config::{self, *}, errors::*};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait HelpersModule:
config::ConfigModule
{
    fn update_rps(&self, stake: &mut Stake<Self::Api>) {
        if stake.remaining_time == 0 {
            return
        }

        let mut current_time = self.blockchain().get_block_timestamp();
        if current_time > stake.end_time {
            current_time = stake.end_time;
        }
        let elapsed_time = current_time - stake.last_rps_update_time;
        if elapsed_time == 0 {
            return
        }

        let staked = stake.staked_amount.clone();
        if staked > 0 {
            let remaining_rewards = stake.remaining_rewards.clone();
            let one_token = BigUint::from(10u64).pow(stake.token_decimals as u32);
            let new_claimable_rewards = match stake.stake_type {
                StakeType::DynamicAPR => {
                    remaining_rewards * elapsed_time / stake.remaining_time
                }
                StakeType::FixedAPR => {
                    &staked * &stake.rewards_per_second * elapsed_time / &one_token
                }
            };
            let new_rps = &new_claimable_rewards * &one_token / staked;
            stake.rps += new_rps;
            stake.claimable_rewards += &new_claimable_rewards;
            require!(stake.remaining_rewards >= new_claimable_rewards, ERROR_OUT_OF_REWARDS);

            stake.remaining_rewards -= new_claimable_rewards;
        }
        stake.last_rps_update_time = current_time;
        stake.remaining_time -= elapsed_time;
    }
}
