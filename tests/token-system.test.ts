import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RwaContract } from "../target/types/rwa_contract";
import { Keypair, SystemProgram } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getMint,
} from "@solana/spl-token";
import { MPL_CORE_PROGRAM_ID } from "@metaplex-foundation/mpl-core";
import { expect } from "chai";

describe("Token System", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RwaContract as Program<RwaContract>;
  const wallet = provider.wallet as anchor.Wallet;

  describe("Fungible Token", () => {
    it("Creates a fungible token with correct decimals and supply", async () => {
      const mintKeypair = Keypair.generate();
      const decimals = 6;
      const supply = 100;

      await program.methods
        .createFungibleToken(decimals, supply)
        .accounts({
          mint: mintKeypair.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([mintKeypair])
        .rpc();

      // Verify the mint was created with correct parameters
      const mintInfo = await getMint(
        provider.connection,
        mintKeypair.publicKey,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );

      expect(mintInfo.decimals).to.equal(decimals);
      expect(mintInfo.supply.toString()).to.equal(
        (supply * 10 ** decimals).toString()
      );
      expect(mintInfo.mintAuthority).to.be.null; // Authority should be revoked
    });
  });

  describe("Non-Fungible Token (NFT)", () => {
    let ftMintKeypair: Keypair;

    before(async () => {
      // Create a fungible token to associate with the NFT
      ftMintKeypair = Keypair.generate();

      await program.methods
        .createFungibleToken(6, 100)
        .accounts({
          mint: ftMintKeypair.publicKey,
          tokenProgram: TOKEN_2022_PROGRAM_ID,
        })
        .signers([ftMintKeypair])
        .rpc();
    });

    it("Creates an NFT with metadata", async () => {
      const assetKeypair = Keypair.generate();

      const args = {
        name: "Property NFT",
        uri: "https://example.com/nft-metadata.json",
      };

      await program.methods
        .createNonFungibleToken(args)
        .accountsPartial({
          payer: wallet.publicKey,
          asset: assetKeypair.publicKey,
          ftMint: ftMintKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          mplCoreProgram: MPL_CORE_PROGRAM_ID,
        })
        .signers([wallet.payer, assetKeypair])
        .rpc();

      // Verify asset state was created
      const [assetStatePda] = anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("asset_state"), assetKeypair.publicKey.toBuffer()],
        program.programId
      );

      const assetState = await program.account.assetState.fetch(assetStatePda);
      expect(assetState.asset.toString()).to.equal(
        assetKeypair.publicKey.toString()
      );
      expect(assetState.ftMint.toString()).to.equal(
        ftMintKeypair.publicKey.toString()
      );
    });
  });
});
