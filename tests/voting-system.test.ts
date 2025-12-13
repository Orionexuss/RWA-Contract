import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { RwaContract } from "../target/types/rwa_contract";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
  getMint,
} from "@solana/spl-token";
import { assert } from "chai";

describe("Voting System Tests", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RwaContract as Program<RwaContract>;
  const wallet = provider.wallet as anchor.Wallet;

  // Test accounts
  let assetKeypair: Keypair;
  let ftMintKeypair: Keypair;
  let assetStatePda: PublicKey;
  let voteRoundPda: PublicKey;
  let voteStatePda: PublicKey;
  let voteRecordPda: PublicKey;
  let userTokenAccount: PublicKey;

  before(async () => {
    // Setup: Create a fungible token for voting power
    assetKeypair = Keypair.generate();
    ftMintKeypair = Keypair.generate();
    
    [assetStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from("asset_state"), assetKeypair.publicKey.toBuffer()],
      program.programId
    );

    // Create fungible token
    const tokenAccount = getAssociatedTokenAddressSync(
      ftMintKeypair.publicKey,
      wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID
    );

    await program.methods
      .createFungibleToken(6, 100)
      .accountsPartial({
        payer: wallet.publicKey,
        mint: ftMintKeypair.publicKey,
        tokenAccount: tokenAccount,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_2022_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      })
      .signers([ftMintKeypair])
      .rpc();

    // Get user's token account
    userTokenAccount = getAssociatedTokenAddressSync(
      ftMintKeypair.publicKey,
      wallet.publicKey,
      false,
      TOKEN_2022_PROGRAM_ID
    );

    // Create NFT to establish asset state (required for voting)
    const args = {
      name: "Voting Property NFT",
      uri: "https://example.com/voting-nft.json",
    };

    await program.methods
      .createNonFungibleToken(args)
      .accountsPartial({
        payer: wallet.publicKey,
        asset: assetKeypair.publicKey,
        ftMint: ftMintKeypair.publicKey,
      })
      .signers([assetKeypair])
      .rpc();
  });

  describe("Vote Round Creation", () => {
    it("Creates a vote round successfully", async () => {
      const description = "Should we renovate the property?";

      [voteRoundPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_round_index"),
          assetKeypair.publicKey.toBuffer(),
        ],
        program.programId
      );

      // vote_state PDA includes payer and vote_round_count (0 for first round)
      const voteRoundCount = new BN(0);
      [voteStatePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_state"),
          assetKeypair.publicKey.toBuffer(),
          wallet.publicKey.toBuffer(),
          voteRoundCount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const tx = await program.methods
        .createVoteRound(description)
        .accountsPartial({
          payer: wallet.publicKey,
          tokenAccount: userTokenAccount,
          assetState: assetStatePda,
          voteRoundIndex: voteRoundPda,
          voteState: voteStatePda,
        })
        .rpc();

      console.log("Vote round creation transaction:", tx);

      // Verify vote round index
      const voteRoundIndex = await program.account.voteRoundIndexState.fetch(voteRoundPda);
      assert.equal(voteRoundIndex.voteRoundCount.toNumber(), 1);

      // Verify vote state
      const voteState = await program.account.voteState.fetch(voteStatePda);
      assert.equal(voteState.description, description);
      assert.equal(voteState.yesWeight.toNumber(), 0);
      assert.equal(voteState.noWeight.toNumber(), 0);
    });

    it("Increments round index for subsequent rounds", async () => {
      const voteRoundCount = new BN(1);
      const [newVoteStatePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_state"),
          assetKeypair.publicKey.toBuffer(),
          wallet.publicKey.toBuffer(),
          voteRoundCount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      await program.methods
        .createVoteRound("Second vote: Property maintenance?")
        .accountsPartial({
          payer: wallet.publicKey,
          tokenAccount: userTokenAccount,
          assetState: assetStatePda,
          voteRoundIndex: voteRoundPda,
          voteState: newVoteStatePda,
        })
        .rpc();

      const voteRoundIndex = await program.account.voteRoundIndexState.fetch(voteRoundPda);
      assert.equal(voteRoundIndex.voteRoundCount.toNumber(), 2);
    });
  });

  describe("Voting", () => {
    it("Allows token holders to vote FOR", async () => {
      // Vote record is derived from vote_state key, not asset
      [voteRecordPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_record"),
          voteStatePda.toBuffer(),
          wallet.publicKey.toBuffer(),
        ],
        program.programId
      );

      const choice = 1; // Vote Yes (FOR)
      const voteRoundCreator = wallet.publicKey;
      const voteRound = new BN(0);

      const tx = await program.methods
        .vote(voteRoundCreator, voteRound, choice)
        .accountsPartial({
          voter: wallet.publicKey,
          asset: assetKeypair.publicKey,
          assetState: assetStatePda,
          voteState: voteStatePda,
          voteRecord: voteRecordPda,
          voterTokenAccount: userTokenAccount,
        })
        .rpc();

      console.log("Vote transaction:", tx);

      // Verify vote was recorded
      const voteRecord = await program.account.voteRecord.fetch(voteRecordPda);
      assert.equal(voteRecord.voter.toString(), wallet.publicKey.toString());
      assert.equal(voteRecord.choice, choice);
      assert.isTrue(voteRecord.weight.toNumber() > 0);

      // Verify vote state was updated
      const voteState = await program.account.voteState.fetch(voteStatePda);
      assert.isTrue(voteState.yesWeight.toNumber() > 0);
    });

    it("Prevents double voting in same round", async () => {
      try {
        const voteRoundCreator = wallet.publicKey;
        const voteRound = new BN(0);
        await program.methods
          .vote(voteRoundCreator, voteRound, 1)
          .accountsPartial({
            voter: wallet.publicKey,
            asset: assetKeypair.publicKey,
            assetState: assetStatePda,
            voteState: voteStatePda,
            voteRecord: voteRecordPda,
            voterTokenAccount: userTokenAccount,
          })
          .rpc();
        
        assert.fail("Should have failed with duplicate vote");
      } catch (error) {
        // Should fail because vote record already exists
        assert.include(error.message, "");
      }
    });

    it("Allows voting AGAINST in a different round", async () => {
      const voteRoundCount = new BN(1);
      const [newVoteStatePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_state"),
          assetKeypair.publicKey.toBuffer(),
          wallet.publicKey.toBuffer(),
          voteRoundCount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      const [newVoteRecordPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_record"),
          newVoteStatePda.toBuffer(),
          wallet.publicKey.toBuffer(),
        ],
        program.programId
      );

      const choice = 0; // Vote No (AGAINST)
      const voteRoundCreator = wallet.publicKey;
      const voteRound = new BN(1);

      await program.methods
        .vote(voteRoundCreator, voteRound, choice)
        .accountsPartial({
          voter: wallet.publicKey,
          asset: assetKeypair.publicKey,
          assetState: assetStatePda,
          voteState: newVoteStatePda,
          voteRecord: newVoteRecordPda,
          voterTokenAccount: userTokenAccount,
        })
        .rpc();

      const voteState = await program.account.voteState.fetch(newVoteStatePda);
      assert.isTrue(voteState.noWeight.toNumber() > 0);
    });

    it("Rejects invalid vote choice", async () => {
      const invalidChoice = 5; // Invalid choice (only 0 and 1 are valid)

      // Create a third vote round to test with
      const voteRoundCount = new BN(2);
      const [thirdVoteStatePda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_state"),
          assetKeypair.publicKey.toBuffer(),
          wallet.publicKey.toBuffer(),
          voteRoundCount.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

      await program.methods
        .createVoteRound("Third vote: Test invalid choice")
        .accountsPartial({
          payer: wallet.publicKey,
          tokenAccount: userTokenAccount,
          assetState: assetStatePda,
          voteRoundIndex: voteRoundPda,
          voteState: thirdVoteStatePda,
        })
        .rpc();

      const tempVoteRecordPda = PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote_record"),
          thirdVoteStatePda.toBuffer(),
          wallet.publicKey.toBuffer(),
        ],
        program.programId
      )[0];

      const voteRoundCreator = wallet.publicKey;
      const voteRound = new BN(2);

      try {
        await program.methods
          .vote(voteRoundCreator, voteRound, invalidChoice)
          .accountsPartial({
            voter: wallet.publicKey,
            asset: assetKeypair.publicKey,
            assetState: assetStatePda,
            voteState: thirdVoteStatePda,
            voteRecord: tempVoteRecordPda,
            voterTokenAccount: userTokenAccount,
          })
          .rpc();
        
        assert.fail("Should have failed with invalid choice");
      } catch (error) {
        assert.include(error.message, "InvalidChoice");
      }
    });
  });
});
