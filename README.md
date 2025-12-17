# Real Estate Tokenization Protocol (Solana)

**This protocol lets you tokenize real estate on Solana using one NFT per property and a fungible token mint that represents fractional ownership.** Holders can transfer fractions, participate in on-chain auctions, and vote on proposals using their tokens.

---

## How It Works

### 1. Property NFT

Each real estate asset is represented by a single NFT that contains the core metadata of the property.

![Image 1](https://github.com/user-attachments/assets/5236e02e-aa8b-4cea-9fa3-1b9e3562fa48)

---

## Fractional Ownership

When a property is tokenized:

- One NFT is minted (the property itself).
- A fungible token mint is created representing economic rights.
- Example: Supply of **20 tokens**, each one equals **5%** ownership.

If Alice transfers 5 tokens to Bob → Bob now owns **25%** of the property’s economic rights.

![Image 2](https://github.com/user-attachments/assets/fe67d7d7-26d6-43fe-a5cf-0a3f964d7d00)

---

## Voting System

Token holders can participate in governance through an on-chain voting mechanism.

- Property owners can create vote rounds with a description.
- Token holders vote FOR (choice 0) or AGAINST (choice 1) based on their token balance.
- Each address can vote once per round.
- Vote weight is proportional to token holdings.

---

## English Auctions

Real estate fractions can be sold via on-chain English auctions.

### Auction Flow

- The property owner (e.g., Alice) starts the auction with a specified end time.
- The contract **locks her fractional tokens**.
- Anyone can bid using USDC, each bid must be higher than the last.
- After the auction end time, anyone can settle the auction.

  - Highest bidder receives the tokens.
  - Alice receives the winning bid amount in USDC.

---

## Current Features

This protocol makes real estate:

- **Fractional** – Divide property ownership into fungible tokens
- **Tradable** – Transfer ownership through token transfers
- **Governable** – Vote on proposals using token-weighted voting
- **Auctionable** – Sell fractions via on-chain English auctions
- Fully managed on Solana

---

## Future Enhancements

Potential features that could be added in future versions:

- **Lending Protocol** – Use fractional tokens as collateral for loans
- **Advanced Auction Types** – Dutch auctions, sealed-bid auctions
- **Dividend Distribution** – Automatic rental income distribution to token holders

---

## Note

The blockchain only manages _digital ownership_. Legal rights over real-world property must be handled through off-chain agreements.
