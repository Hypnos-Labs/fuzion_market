# Juno Vaults

### [Architecture Overview](#-architecture-overview)
### [Changelog](CHANGELOG.md)

**Contributors**
- [LeTurt](https://twitter.com/leturt_)
- [Reecebcups](https://twitter.com/Reecepbcups_)

# Architecture Overview

The Juno Vaults contract allows 2 wallets to trade escrows of tokens between eachother in a trustless manner

The escrows can contain any number of native, cw20, or cw721 tokens

</br>

# High Level

Say we have 2 users:
> `Sam` - the seller</br>

> `John` - the buyer

---

## Sam wants to sell

Sam wants to sell `5 JUNO` + `10 CW20` + `Dog NFT #3`

At a price of `10 OSMO` + `Cat NFT #8`

In order, Sam will
- Create a listing with `ExecuteMsg::CreateListing`, sending `5 JUNO` along with the message
- Send `10 CW20` + `Dog NFT #3` to the contract, specifying the Listing ID in each message
- Call `ExecuteMsg::Finalize`, specifying an `Expiration`
  - Once `Finalize` has been called, the listing can be purchased and Sam can no longer make changes
  - If the listing is not purchased before `Expiration`, Sam can call `ExecuteMsg::Refund` to remove the funds + delete the Listing

![image](https://user-images.githubusercontent.com/89463679/210180396-c7153b07-30c3-4682-b556-d75df3050d8f.png)

---

## John wants to buy

John sees this listing and wants to buy it

In order, John will
- Create a bucket with `ExecuteMsg::CreateListing`, sending `10 OSMO` along with the message
- Send `Cat NFT #8` to the contract, specifying the Bucket ID in the message
- Call `ExecuteMsg::BuyListing`, specifying the listing ID being purchased (`listing_id: 1`), and the bucket ID to use (`bucket_id: 1`)

If the assets in `Bucket #1` match the price of `Listing #1`, ownership of each will be traded between them 

![image](https://user-images.githubusercontent.com/89463679/210180678-6b1ed2c9-1b7a-4809-be18-000972d2124c.png)

![image](https://user-images.githubusercontent.com/89463679/210180798-2c463f29-2d55-497b-b73c-3d5204509e76.png)

---

## Sam and John remove their proceeds

Now that the trade is complete, both Sam and John can remove their proceeds from the contract

Sam can now remove sale proceeds by calling `ExecuteMsg::RemoveBucket {bucket_id: 1}`

John can now remove purchase proceeds by calling `ExecuteMsg::WithdrawPurchased {listing_id: 1}`

![image](https://user-images.githubusercontent.com/89463679/210180897-910546c0-7a82-4c09-a1bf-a1b8bb1136b5.png)

---
---

## Attributions

- [Juno](https://junonetwork.io)
- [CosmWasm](https://github.com/cosmwasm)

