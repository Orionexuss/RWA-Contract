# Real Estate Tokenization Protocol (Solana)

**This protocol lets you tokenize real estate on Solana using one NFT per property and a fungible token mint that represents fractional ownership.** Holders can transfer fractions, use them as collateral for loans, and participate in on-chain auctions.

---

## How It Works

### 1. Property NFT

Each real estate asset is represented by a single NFT that contains the core metadata of the property.

![Image 1](https://github.com/user-attachments/assets/5236e02e-aa8b-4cea-9fa3-1b9e3562fa48)


---

## Fractional Ownership

When a property is tokenized:

* One NFT is minted (the property itself).
* A fungible token mint is created representing economic rights.
* Example: Supply of **20 tokens**, each one equals **5%** ownership.

If Alice transfers 5 tokens to Bob → Bob now owns **25%** of the property’s economic rights.

![Image 2](https://github.com/user-attachments/assets/fe67d7d7-26d6-43fe-a5cf-0a3f964d7d00)


---

## Borrowing Against Your Tokens

Owners can lock their fractional tokens as collateral and borrow against their value.

* Deposit tokens → get a loan.
* Repay loan → unlock tokens.
* If collateral value falls too much, liquidation may happen.

---

## English Auctions

Real estate fractions can be sold via a **7‑day English auction**.

### Auction Flow

* The property owner (e.g., Alice) starts the auction.
* The contract **locks her fractional tokens**.
* Anyone can bid, each bid must be higher than the last.
* All bidders except the current highest bidder may withdraw their bids.
* After 7 days, anyone can finalize the auction.

  * Highest bidder receives the tokens.
  * Alice receives the winning bid amount.

---

## Purpose

This protocol makes real estate:

* Fractional
* Tradable
* Collateralizable
* Auctionable
* Fully managed on Solana

---

## Note

The blockchain only manages *digital ownership*. Legal rights over real-world property must be handled through off-chain agreements.

---

## Development Status

This protocol is **still under active development**, and features or behavior may change as the project evolves.
