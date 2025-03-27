multiversx_sc::imports!();

use crate::{common::errors::*, helpers};

use super::storage::*;

#[multiversx_sc::module]
pub trait ConfigModule:
super::storage::StorageModule
+helpers::HelpersModule
{
    // state
    #[view(getState)]
    #[storage_mapper("state")]
    fn state(&self) -> SingleValueMapper<State>;

    #[only_owner]
    #[endpoint(setStateActive)]
    fn set_state_active(&self) {
        self.state().set(State::Active);
    }

    #[only_owner]
    #[endpoint(setStateInactive)]
    fn set_state_inactive(&self) {
        self.state().set(State::Inactive);
    }

    // stakes
    #[view(getStake)]
    #[storage_mapper("stakes")]
    fn stake(&self, id: u64) -> SingleValueMapper<Stake<Self::Api>>;

    #[view(getLastStakeId)]
    #[storage_mapper("last_stake_id")]
    fn last_stake_id(&self) -> SingleValueMapper<u64>;

    #[view(getStakeByToken)]
    fn get_stake_by_token(&self, token: &TokenIdentifier) -> Option<Stake<Self::Api>> {
        for id in 0..self.last_stake_id().get() {
            if self.stake(id).is_empty() {
                continue;
            }

            let stake = self.stake(id).get();
            if &stake.token == token {
                return Some(stake);
            }
        }

        None
    }

    #[view(getStakeByLiquidToken)]
    fn get_stake_by_liquid_token(&self, token: &TokenIdentifier) -> Option<Stake<Self::Api>> {
        for id in 0..self.last_stake_id().get() {
            if self.stake(id).is_empty() {
                continue;
            }

            let stake = self.stake(id).get();
            if &stake.liquid_token == token {
                return Some(stake);
            }
        }

        None
    }

    #[view(getStakes)]
    fn get_stakes(&self) -> ManagedVec<Stake<Self::Api>> {
        let mut stakes = ManagedVec::new();
        for id in 0..self.last_stake_id().get() {
            if self.stake(id).is_empty() {
                continue;
            }

            let mut stake = self.stake(id).get();
            self.update_rps(&mut stake);
            stakes.push(self.stake(id).get());
        }

        stakes
    }

    #[view(getUserRewards)]
    fn get_user_rewards(&self, id: u64, staked_tokens: ManagedVec<EsdtTokenPayment>) -> BigUint {
        require!(!self.stake(id).is_empty(), ERROR_STAKE_NOT_FOUND);

        let mut stake = self.stake(id).get();
        self.update_rps(&mut stake);
    
        let mut total_rewards = BigUint::zero();
        let one_token = BigUint::from(10u64).pow(stake.token_decimals as u32);
        for payment in staked_tokens.iter() {
            require!(payment.token_identifier == stake.liquid_token, ERROR_WRONG_PAYMENT_TOKEN);

            let attributes: StakeTokenAttributes<Self::Api> = self.blockchain().get_token_attributes(&stake.liquid_token, payment.token_nonce);
            total_rewards += &payment.amount * &(&stake.rps - &attributes.rps) / &one_token;
        }

        total_rewards
    }
}
