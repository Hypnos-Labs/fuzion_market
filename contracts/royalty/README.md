# Royalty Regsitry Contract for Fuzion Market

NFT Creators can register their collections here if they'd like to receive royalties on NFTs traded on the platform

## Quick start for Registering

- Registrant **must** be the Admin of the NFT contract being registered

- Max royalty amount is 300 bps (3%)

- Register your collection with this ExecuteMsg
```
    Register {
        // The NFT contract you're registering
        nft_contract: String,
        // The address for royalties to be sent to
        payout_addr: String,
        // Royalty amount
        bps: u64,
    },
```

## Introduction to how Royalties work
**bps** (or bips) is the percentage of royalties that you want to charge

- `bps = 1` : 0.01% royalties
- `bps = 100` : 1% royalties
- `bps = 300` : 3% royalties (maximum)

Royalites are paid out in whatever tokens (Native or CW20) are involved in the trade, depending on if they're being sold or purchased. See **Some things to note** for further details

Example Trade **assuming the current FeeDenom is USDC**

**Listing Seller is selling**
- `Crazy Cows #69` (registered with 1% / 100bps royalty)
- `Mad Dogs #420` (registered with 1% / 100bps royalty)
- `200 JUNO`
- `100 USDC`

**Buyer is paying**
- `Crazy Cows #1` (registered with 1% / 100bps royalty)
- `Sleepy Bats #19` (registered with 1% / 100bps royalty)
- `200 STARS`
- `100 OSMO`

---

**Community Pool Fee**: `0.5 USDC`

**Royalties for Crazy Cows**: `2 STARS` + `1 OSMO` + `2 JUNO` + `0.995 USDC`

**Royalties for Mad Dogs**: `2 STARS` + `1 OSMO`

**Royalties for Sleepy Bats**: `2 JUNO` + `0.995 USDC`

**Listing Seller receives**: `Crazy Cows #1` + `Sleepy Bats #19` + `196 STARS` + `98 OSMO`

**Buyer receives**: `Crazy Cows #69` + `Mad Dogs #420` + `196 JUNO` + `97.51 USDC`




## **Some things to note**

- Maximum of 3% royalties per collection
- Trades on Fuzion that contain more than 1 collection NFT do not pay out duplicate royalties
- Royalties are paid out from all tokens involved in the corresponding side of the trade
- A Seller pays royalties from their proceeds
- A Buyer pays royalties from what they're purchasing
- Royalties are capped at 50% of a trade's purchase or sale price (requires 17 NFTs from different collections on a single side of the trade)
  - If the royalty amount is over 50%, the Listing will simply not be purchasable & the seller will have to delete the Listing & recreate it




