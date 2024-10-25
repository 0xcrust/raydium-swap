use solana_sdk::pubkey::Pubkey;

#[derive(Copy, Clone, Debug, Default)]
pub enum ComputeUnitLimits {
    #[default]
    Dynamic,
    Fixed(u64),
}

#[derive(Copy, Clone, Debug)]
pub enum PriorityFeeConfig {
    DynamicMultiplier(u64),
    FixedCuPrice(u64),
    JitoTip(u64),
}

#[derive(Copy, Clone, Debug)]
pub struct SwapConfig {
    pub priority_fee: Option<PriorityFeeConfig>,
    pub cu_limits: Option<ComputeUnitLimits>,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub as_legacy_transaction: Option<bool>,
}

#[derive(Clone, Debug, Default)]
pub struct SwapConfigOverrides {
    pub priority_fee: Option<PriorityFeeConfig>,
    pub cu_limits: Option<ComputeUnitLimits>,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub destination_token_account: Option<Pubkey>,
    pub as_legacy_transaction: Option<bool>,
}

#[derive(Copy, Clone, Debug)]
pub struct SwapInput {
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub slippage_bps: u16,
    pub amount: u64,
    pub mode: SwapExecutionMode,
    pub market: Option<Pubkey>,
}

#[derive(Copy, Clone, Debug)]
pub enum SwapExecutionMode {
    ExactIn,
    ExactOut,
}
impl SwapExecutionMode {
    pub fn amount_specified_is_input(&self) -> bool {
        matches!(self, SwapExecutionMode::ExactIn)
    }
}
