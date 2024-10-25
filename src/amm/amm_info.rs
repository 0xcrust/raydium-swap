use bytemuck::{Pod, Zeroable};
use raydium_amm::state::Loadable;
use safe_transmute::trivial::TriviallyTransmutable;
use solana_program::pubkey::Pubkey;

macro_rules! impl_loadable {
    ($type_name:ident) => {
        unsafe impl Zeroable for $type_name {}
        unsafe impl Pod for $type_name {}
        unsafe impl TriviallyTransmutable for $type_name {}
        impl Loadable for $type_name {}
    };
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AmmInfo {
    /// Initialized status.
    pub status: u64,
    /// Nonce used in program address.
    /// The program address is created deterministically with the nonce,
    /// amm program id, and amm account pubkey.  This program address has
    /// authority over the amm's token coin account, token pc account, and pool
    /// token mint.
    pub nonce: u64,
    /// max order count
    pub order_num: u64,
    /// within this range, 5 => 5% range
    pub depth: u64,
    /// coin decimal
    pub coin_decimals: u64,
    /// pc decimal
    pub pc_decimals: u64,
    /// amm machine state
    pub state: u64,
    /// amm reset_flag
    pub reset_flag: u64,
    /// min size 1->0.000001
    pub min_size: u64,
    /// vol_max_cut_ratio numerator, sys_decimal_value as denominator
    pub vol_max_cut_ratio: u64,
    /// amount wave numerator, sys_decimal_value as denominator
    pub amount_wave: u64,
    /// coinLotSize 1 -> 0.000001
    pub coin_lot_size: u64,
    /// pcLotSize 1 -> 0.000001
    pub pc_lot_size: u64,
    /// min_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub min_price_multiplier: u64,
    /// max_cur_price: (2 * amm.order_num * amm.pc_lot_size) * max_price_multiplier
    pub max_price_multiplier: u64,
    /// system decimal value, used to normalize the value of coin and pc amount
    pub sys_decimal_value: u64,
    /// All fee information
    pub fees: raydium_amm::state::Fees,
    /// Statistical data
    pub state_data: StateData,
    /// Coin vault
    pub coin_vault: Pubkey,
    /// Pc vault
    pub pc_vault: Pubkey,
    /// Coin vault mint
    pub coin_vault_mint: Pubkey,
    /// Pc vault mint
    pub pc_vault_mint: Pubkey,
    /// lp mint
    pub lp_mint: Pubkey,
    /// open_orders key
    pub open_orders: Pubkey,
    /// market key
    pub market: Pubkey,
    /// market program key
    pub market_program: Pubkey,
    /// target_orders key
    pub target_orders: Pubkey,
    /// padding
    pub padding1: [u64; 8],
    /// amm owner key
    pub amm_owner: Pubkey,
    /// pool lp amount
    pub lp_amount: u64,
    /// client order id
    pub client_order_id: u64,
    /// padding
    pub padding2: [u64; 2],
}
impl_loadable!(AmmInfo);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StateData {
    /// delay to take pnl coin
    pub need_take_pnl_coin: u64,
    /// delay to take pnl pc
    pub need_take_pnl_pc: u64,
    /// total pnl pc
    pub total_pnl_pc: u64,
    /// total pnl coin
    pub total_pnl_coin: u64,
    /// ido pool open time
    pub pool_open_time: u64,
    /// padding for future updates
    pub padding: [u64; 2],
    /// switch from orderbookonly to init
    pub orderbook_to_init_time: u64,

    /// swap coin in amount
    pub swap_coin_in_amount: [u8; 16],
    /// swap pc out amount
    pub swap_pc_out_amount: [u8; 16],
    /// charge pc as swap fee while swap pc to coin
    pub swap_acc_pc_fee: u64,

    /// swap pc in amount
    pub swap_pc_in_amount: [u8; 16],
    /// swap coin out amount
    pub swap_coin_out_amount: [u8; 16],
    /// charge coin as swap fee while swap coin to pc
    pub swap_acc_coin_fee: u64,
}

impl From<StateData> for raydium_amm::state::StateData {
    fn from(value: StateData) -> Self {
        Self {
            need_take_pnl_coin: value.need_take_pnl_coin,
            need_take_pnl_pc: value.need_take_pnl_pc,
            total_pnl_coin: value.total_pnl_coin,
            total_pnl_pc: value.total_pnl_pc,
            pool_open_time: value.pool_open_time,
            padding: value.padding,
            orderbook_to_init_time: value.orderbook_to_init_time,
            swap_acc_pc_fee: value.swap_acc_pc_fee,
            swap_acc_coin_fee: value.swap_acc_coin_fee,
            swap_coin_in_amount: u128::from_le_bytes(value.swap_coin_in_amount),
            swap_pc_out_amount: u128::from_le_bytes(value.swap_pc_out_amount),
            swap_pc_in_amount: u128::from_le_bytes(value.swap_pc_in_amount),
            swap_coin_out_amount: u128::from_le_bytes(value.swap_coin_out_amount),
        }
    }
}

impl From<AmmInfo> for raydium_amm::state::AmmInfo {
    fn from(value: AmmInfo) -> Self {
        raydium_amm::state::AmmInfo {
            status: value.status,
            nonce: value.nonce,
            order_num: value.order_num,
            depth: value.depth,
            coin_decimals: value.coin_decimals,
            pc_decimals: value.pc_decimals,
            state: value.state,
            reset_flag: value.reset_flag,
            min_size: value.min_size,
            vol_max_cut_ratio: value.vol_max_cut_ratio,
            amount_wave: value.amount_wave,
            coin_lot_size: value.coin_lot_size,
            pc_lot_size: value.pc_lot_size,
            min_price_multiplier: value.min_price_multiplier,
            max_price_multiplier: value.max_price_multiplier,
            sys_decimal_value: value.sys_decimal_value,
            fees: value.fees,
            state_data: value.state_data.into(),
            coin_vault: value.coin_vault,
            pc_vault: value.pc_vault,
            coin_vault_mint: value.coin_vault_mint,
            pc_vault_mint: value.pc_vault_mint,
            lp_mint: value.lp_mint,
            open_orders: value.open_orders,
            market: value.market,
            market_program: value.market_program,
            target_orders: value.target_orders,
            padding1: value.padding1,
            amm_owner: value.amm_owner,
            lp_amount: value.lp_amount,
            client_order_id: value.client_order_id,
            padding2: value.padding2,
        }
    }
}
