multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum State {
    Inactive,
    Active,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Copy, Clone, Debug)]
pub enum StakeType {
    FixedAPR,
    DynamicAPR,
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
pub struct Stake<M: ManagedTypeApi> {
    pub id: u64,
    pub state: State,
    pub stake_type: StakeType,
    pub token: TokenIdentifier<M>,
    pub token_decimals: u8,
    pub liquid_token: TokenIdentifier<M>,
    pub reward_token: TokenIdentifier<M>,
    pub staked_amount: BigUint<M>,
    pub rewards_amount: BigUint<M>,
    pub claimable_rewards: BigUint<M>,
    pub remaining_rewards: BigUint<M>,
    pub rewards_per_second: BigUint<M>, // apr
    pub start_time: u64,
    pub end_time: u64,
    pub remaining_time: u64,
    pub rps: BigUint<M>,
    pub last_rps_update_time: u64,
}

impl<M> Stake<M>
where M: ManagedTypeApi {
    pub fn is_active(&self, current_time: u64) -> bool {
        self.state == State::Active &&
        self.end_time > current_time &&
        self.remaining_rewards > 0 &&
        self.remaining_rewards >= self.claimable_rewards
    }
}

#[type_abi]
#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Eq, Clone, Debug)]
pub struct StakeTokenAttributes<M: ManagedTypeApi> {
    pub rps: BigUint<M>,
}

#[multiversx_sc::module]
pub trait StorageModule {
}
