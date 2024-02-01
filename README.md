## Raydium-Swap
A CLI tool to interact with Raydium pools and make swaps.

Each swap is automatically made against SOL. I.e USDC/SOL or SOL/BONK, but not USDC/BONK.

## Prerequisites
Rust and Cargo [installation](https://www.rust-lang.org/tools/install).

## Usage
Run `cargo run help`:
```sh
Usage: swap [CLUSTER] <COMMAND>

Commands:
  swap           Perform a swap against wSOL
  simulate-swap  TODO: Simulate a WSOL swap
  fetch-pools    Dump pool details from `https://api.raydium.io/v2/sdk/liquidity/mainnet.json`
  fetch-prices   Fetch token prices from `https://api.raydium.io/v2/main/price`
  get-price-usd  Gets the price of a token in USD
  get-price-sol  Gets the price of a token in SOL
  help           Print this message or the help of the given subcommand(s)

Arguments:
  [CLUSTER]  URL for Solana's JSON RPC or moniker (or their first letter): [mainnet-beta,
                 testnet, devnet, localhost] [default: mainnet]
```

or `cargo run <Command> --help` for a specific command. e,g `cargo run swap --help`:
```sh
Usage: swap swap [OPTIONS] --keypair <KEYPAIR> --in-token <IN_TOKEN> --out-token <OUT_TOKEN> --amount-in <AMOUNT_IN>

Options:
      --keypair <KEYPAIR>
          Path to the provider keypair file
      --in-token <IN_TOKEN>
          Pubkey of the input token mint
      --out-token <OUT_TOKEN>
          Pubkey of the output token mint
      --amount-in <AMOUNT_IN>
          Amount of input tokens provided by the user for a swap.
      --fee-percentage <FEE_PERCENTAGE>
          The (optional) percentage charged as fee on each trade
      --fee-vault <FEE_VAULT>
          The (optional) vault fee tokens are sent to
      --pool-cache <POOL_CACHE>
          The (optional) path to the json file that stores information on raydium pools
      --price-cache <PRICE_CACHE>
          The (optional) Path to the file that stores information on token prices
      --slippage <SLIPPAGE>
          The (optional) slippage tolerance percentage. Default is 0.5% [default: 0.5]
      --allow-unofficial <ALLOW_UNOFFICIAL>
          (Optional) Allow interactions with non-official Raydium pools. Default is false [possible values: true, false]
  -h, --help
```

Example:
```sh
RUST_LOG=info cargo run swap --keypair ./provider.json --in-token EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v --out-token So11111111111111111111111111111111111111112 --amount-in 1
```

## Notes
This codebase relies on API requests to Raydium endpoint to fetch the correct liquidity-pool addresses for a swap and 
the price of tokens. This information is saved to  `pool-cache`(for lp-addresses) and `price-cache`(for token prices) 
json files. Subsequent runs can specify these files to prevent having to make (relatively) time-wasting calls to 
retrieve the same data.

The pool-cache information is saved to `./pools.json` while the price-cache information is saved to `./prices.json`. 

The `pool-cache` should rarely need to be updated for most program runs. However, price is constantly changing and should be
updated more frequently. The request to fetch price-information takes much less time than for lp-information.

The `allow-unofficial` argument defaults to `false` which limits the number of available pools for a swap. To avoid this, 
explicitly pass in this argument with a `true` value.

