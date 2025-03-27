multiversx_sc::imports!();

use crate::{common::{config, errors::*, storage::{self, *}}, helpers};

#[multiversx_sc::module]
pub trait UserModule:
storage::StorageModule
+config::ConfigModule
+helpers::HelpersModule
{
    #[payable("*")]
    #[endpoint(userStake)]
    fn user_stake(&self) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);

        let payment = self.call_value().single_esdt();
        let mut stake = match self.get_stake_by_token(&payment.token_identifier) {
            Some(stake) => stake,
            None => {
                sc_panic!(ERROR_WRONG_PAYMENT_TOKEN);
            }
        };

        let current_time = self.blockchain().get_block_timestamp();
        if stake.start_time == 0 {
            stake.start_time = current_time;
            stake.last_rps_update_time = current_time;
            stake.remaining_time = stake.end_time - current_time;
        }
        require!(self.update_rps(&mut stake), ERROR_OUT_OF_REWARDS);
        require!(stake.is_active(current_time), ERROR_STAKE_INACTIVE);

        let attributes = StakeTokenAttributes {
            rps: stake.rps.clone(),
        };
        let nonce = self.send().esdt_nft_create_compact(&stake.liquid_token, &payment.amount, &attributes);
        self.send().direct_esdt(&self.blockchain().get_caller(), &stake.liquid_token, nonce, &payment.amount);
        stake.staked_amount += payment.amount;
        self.stake(stake.id).set(stake);
    }

    #[payable("*")]
    #[endpoint(userUnstake)]
    fn user_unstake(&self) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);

        let payments = self.call_value().all_esdt_transfers();
        require!(payments.len() > 0, ERROR_WRONG_PAYMENT_TOKEN);

        let payment_token = payments.get(0).token_identifier;
        for payment in payments.iter() {
            require!(payment.token_identifier == payment_token, ERROR_WRONG_PAYMENT_TOKEN);
        }
        let mut stake = match self.get_stake_by_liquid_token(&payment_token) {
            Some(stake) => stake,
            None => {
                sc_panic!(ERROR_WRONG_PAYMENT_TOKEN);
            }
        };
        require!(stake.state == State::Active, ERROR_STAKE_INACTIVE);
        require!(self.update_rps(&mut stake), ERROR_OUT_OF_REWARDS);

        let mut total_unstake_amount = BigUint::zero();
        let mut total_rewards = BigUint::zero();
        let caller = self.blockchain().get_caller();
        let one_token = BigUint::from(10u64).pow(stake.token_decimals as u32);
        for payment in payments.iter() {
            total_unstake_amount += &payment.amount;
            let attributes: StakeTokenAttributes<Self::Api> = self.blockchain().get_token_attributes(&stake.liquid_token, payment.token_nonce);
            total_rewards += &payment.amount * &(&stake.rps - &attributes.rps) / &one_token;
            self.send().esdt_local_burn(&stake.liquid_token, payment.token_nonce, &payment.amount);
        }
        self.send().direct_esdt(&caller, &stake.token, 0, &total_unstake_amount);
        if total_rewards > 0 {
            self.send().direct_esdt(&caller, &stake.reward_token, 0, &total_rewards);
        }
        stake.staked_amount -= total_unstake_amount;
        self.stake(stake.id).set(stake);
    }

    #[payable("*")]
    #[endpoint(claimRewards)]
    fn claim_rewards(&self) {
        require!(self.state().get() == State::Active, ERROR_CONTRACT_INACTIVE);

        let payments = self.call_value().all_esdt_transfers();
        require!(payments.len() > 0, ERROR_WRONG_PAYMENT_TOKEN);

        let payment_token = payments.get(0).token_identifier;
        for payment in payments.iter() {
            require!(payment.token_identifier == payment_token, ERROR_WRONG_PAYMENT_TOKEN);
        }
        let mut stake = match self.get_stake_by_liquid_token(&payment_token) {
            Some(stake) => stake,
            None => {
                sc_panic!(ERROR_WRONG_PAYMENT_TOKEN);
            }
        };
        require!(stake.state == State::Active, ERROR_STAKE_INACTIVE);

        self.update_rps(&mut stake);
        let mut total_claimed_amount = BigUint::zero();
        let mut total_rewards = BigUint::zero();
        let caller = self.blockchain().get_caller();
        for payment in payments.iter() {
            total_claimed_amount += &payment.amount;
            let attributes: StakeTokenAttributes<Self::Api> = self.blockchain().get_token_attributes(&stake.liquid_token, payment.token_nonce);
            total_rewards += &payment.amount * &(&stake.rps - &attributes.rps) / BigUint::from(10u64).pow(stake.token_decimals as u32);
            self.send().esdt_local_burn(&stake.liquid_token, payment.token_nonce, &payment.amount);
        }
        let attributes = StakeTokenAttributes {
            rps: stake.rps.clone(),
        };
        let nonce = self.send().esdt_nft_create_compact(&stake.liquid_token, &total_claimed_amount, &attributes);
        self.send().direct_esdt(&caller, &stake.liquid_token, nonce, &total_claimed_amount);
        if total_rewards > 0 {
            self.send().direct_esdt(&caller, &stake.reward_token, 0, &total_rewards);
        }
        self.stake(stake.id).set(stake);
    }
}
