import {
  MPL_TOKEN_METADATA_PROGRAM_ID,
  TokenStandard,
  createNft,
  delegateStandardV1,
  findMasterEditionPda,
  findMetadataPda,
  lockV1,
  mplTokenMetadata,
  verifyCollectionV1,
} from "@metaplex-foundation/mpl-token-metadata";
import {
  generateSigner,
  keypairIdentity,
  percentAmount,
  publicKey,
} from "@metaplex-foundation/umi";

import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  ComputeBudgetProgram,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionMessage,
  TransactionSignature,
  VersionedTransaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  fromWeb3JsKeypair,
  fromWeb3JsPublicKey,
  toWeb3JsKeypair,
  toWeb3JsPublicKey,
} from "@metaplex-foundation/umi-web3js-adapters";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount,
  transfer,
  NATIVE_MINT,
  getOrCreateAssociatedTokenAccount,
  createSyncNativeInstruction,
} from "@solana/spl-token";
import { step, xstep } from "mocha-steps";
import {
  ClmmPoolInfo,
  DEVNET_PROGRAM_ID,
  Liquidity,
  MarketV2,
  RENT_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  TxVersion,
  WSOL,
  buildSimpleTransaction,
} from "@raydium-io/raydium-sdk";
import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { assert } from "chai";
import {
  getOrCreateAssociatedTokenAccountInstruction,
  getSimulationUnits,
} from "./utils";
import { SafePresale } from "../target/types/safe_presale";

describe("Safe Presale", () => {
  // Configure the client to use the local cluster.
  // Use the RPC endpoint of your choice.
  anchor.setProvider(anchor.AnchorProvider.env());

  //
  // Program APIs.
  //

  const program = anchor.workspace.SafePresale as Program<SafePresale>;
  const umi = createUmi(program.provider.connection.rpcEndpoint);
  //randomly generated keypair
  const signer = umi.eddsa.createKeypairFromSecretKey(
    Buffer.from([
      225, 66, 240, 160, 100, 176, 216, 156, 98, 248, 136, 34, 108, 179, 97, 33,
      245, 103, 165, 252, 153, 131, 20, 190, 60, 85, 11, 240, 176, 184, 50, 183,
      208, 37, 214, 8, 236, 36, 232, 48, 167, 48, 193, 156, 104, 55, 81, 126,
      209, 94, 147, 84, 22, 209, 65, 127, 206, 246, 2, 145, 207, 168, 186, 29,
    ])
  );
  umi.use(mplTokenMetadata()).use(keypairIdentity(signer));
  console.log(signer.publicKey);

  let poolId: PublicKey;
  let rewardMint: {
    mint: PublicKey;
    name: string;
    symbol: string;
    decimal: number;
    uri: string;
  };
  let purchaseReceipt: PublicKey;
  let ammInfo: {
    marketId: PublicKey;
    requestQueue: PublicKey;
    eventQueue: PublicKey;
    bids: PublicKey;
    asks: PublicKey;
    baseVault: PublicKey;
    quoteVault: PublicKey;
    baseMint: PublicKey;
    quoteMint: PublicKey;
  };

  //
  // NFTs. These are the two mad lads for the tests.
  //
  let nftA: {
    tokenAddress: PublicKey;
    mintAddress: PublicKey;
    masterEditionAddress: PublicKey;
    metadataAddress: PublicKey;
  };
  let nftB: {
    mintAddress: PublicKey;
    masterEditionAddress: PublicKey;
    metadataAddress: PublicKey;
  };
  let collection: {
    mintAddress: PublicKey;
    masterEditionAddress: PublicKey;
    metadataAddress: PublicKey;
  };

  step(
    "Setup: creates one nft, verified as part of the same collection",
    async () => {
      // let madLadCollection = generateSigner(umi);
      // await createNft(umi, {
      //   mint: madLadCollection,
      //   name: "Mad Lad Collection",
      //   uri: "https://example.com/my-collection.json",
      //   sellerFeeBasisPoints: percentAmount(5.5), // 5.5%
      //   creators: [{ address: signer.publicKey, verified: true, share: 100 }],
      //   isCollection: true,
      // }).sendAndConfirm(umi);
      collection = {
        mintAddress: new PublicKey(
          "C7MU51LL1EJmnD7QhXC1ejpp6ZgSS79kS6rkd7PCYvXN"
        ),
        masterEditionAddress: toWeb3JsPublicKey(
          findMasterEditionPda(umi, {
            mint: publicKey("C7MU51LL1EJmnD7QhXC1ejpp6ZgSS79kS6rkd7PCYvXN"),
          })[0]
        ),
        metadataAddress: toWeb3JsPublicKey(
          findMetadataPda(umi, {
            mint: publicKey("C7MU51LL1EJmnD7QhXC1ejpp6ZgSS79kS6rkd7PCYvXN"),
          })[0]
        ),
      };
      // let madlad1 = generateSigner(umi);
      // await createNft(umi, {
      //   mint: madlad1,
      //   name: "MadLad 1",
      //   uri: "https://arweave.net/my-content-hash",
      //   sellerFeeBasisPoints: percentAmount(5.5), // 5.5%
      //   isMutable: true,
      //   collection: {
      //     key: madLadCollection.publicKey,
      //     verified: false,
      //   },
      // }).sendAndConfirm(umi);
      nftA = {
        mintAddress: new PublicKey(
          "4HEkZSUaGDEwCMDtxowi6RCCsUyY4LAaFzJBE4UeCpxM"
        ),
        tokenAddress: getAssociatedTokenAddressSync(
          new PublicKey("4HEkZSUaGDEwCMDtxowi6RCCsUyY4LAaFzJBE4UeCpxM"),
          toWeb3JsPublicKey(signer.publicKey),
          true
        ),
        masterEditionAddress: toWeb3JsPublicKey(
          findMasterEditionPda(umi, {
            mint: publicKey("4HEkZSUaGDEwCMDtxowi6RCCsUyY4LAaFzJBE4UeCpxM"),
          })[0]
        ),
        metadataAddress: toWeb3JsPublicKey(
          findMetadataPda(umi, {
            mint: publicKey("4HEkZSUaGDEwCMDtxowi6RCCsUyY4LAaFzJBE4UeCpxM"),
          })[0]
        ),
      };
      // await verifyCollectionV1(umi, {
      //   metadata: findMetadataPda(umi, { mint: madlad1.publicKey }),
      //   collectionMint: madLadCollection.publicKey,
      //   authority: umi.payer,
      // }).sendAndConfirm(umi);

      // await delegateStandardV1(umi, {
      //   mint: fromWeb3JsPublicKey(nftA.mintAddress),
      //   tokenOwner: signer.publicKey,
      //   authority: umi.payer,
      //   delegate: signer.publicKey,
      //   tokenStandard: TokenStandard.NonFungible,
      // }).sendAndConfirm(umi);
      // const account = await getAccount(
      //   program.provider.connection,
      //   nftA.tokenAddress
      // );
      // console.log(account);

      // await lockV1(umi, {
      //   mint: fromWeb3JsPublicKey(nftA.mintAddress),
      //   authority: umi.payer,
      //   tokenStandard: TokenStandard.NonFungible,
      // }).sendAndConfirm(umi);
    }
  );

  function generateRandomU64() {
    // Generate two 32-bit integers
    const upper = Math.floor(Math.random() * 0x100000000); // 2^32
    const lower = Math.floor(Math.random() * 0x100000000); // 2^32

    // Combine them to form a 64-bit integer
    const u64 = (upper << 32) | lower;

    return u64;
  }

  step("Initialize a pool", async () => {
    const randomKey = generateRandomU64();
    const [rewardMintKey] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), new BN(randomKey).toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    [poolId] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), rewardMintKey.toBuffer()],
      program.programId
    );

    rewardMint = {
      mint: rewardMintKey,
      name: "XYZ",
      symbol: "Fock",
      decimal: 5,
      uri: "https://www.madlads.com/mad_lads_logo.svg",
    };
    const liquidityPoolSupply = new BN(300000000);
    const initialSupply = new BN(700000000);
    const presaleTarget = new BN(LAMPORTS_PER_SOL * 0.5);
    const vestingPeriod = 3 * 24 * 60 * 60; //3days in seconds
    const presaleDuration = 30; // in seconds

    const [rewardMint_metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID).toBuffer(),
        rewardMint.mint.toBuffer(),
      ],
      toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID)
    );

    const poolAndMintRewardAta = getAssociatedTokenAddressSync(
      rewardMint.mint,
      poolId,
      true
    );

    try {
      await program.methods
        .initPool({
          quoteMint: NATIVE_MINT,
          name: rewardMint.name,
          symbol: rewardMint.symbol,
          decimals: rewardMint.decimal,
          uri: rewardMint.uri,
          presaleTarget: presaleTarget,
          creatorFeeBasisPoints: 500,
          delegate: null,
          maxAmountPerPurchase: new BN(LAMPORTS_PER_SOL),
          vestingPeriod: vestingPeriod,
          liquidityPoolSupply: liquidityPoolSupply,
          initialSupply: initialSupply,
          presaleDuration: presaleDuration,
          randomKey: new BN(randomKey),
          requiresCollection: false,
        })
        .accounts({
          payer: signer.publicKey,
          rewardMintMetadata: rewardMint_metadata,
          poolRewardMintTokenAccount: poolAndMintRewardAta,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
    } catch (e) {
      console.log(e);
    }
    const data = await program.account.pool.fetch(poolId);
    assert(
      data.authority.toBase58() === signer.publicKey.toString(),
      "Wrong authority"
    );
    assert(
      data.liquidityCollected.toNumber() === 0,
      "Initial Liquidity is not zero"
    );
    assert(
      data.mint.toBase58() === rewardMint.mint.toBase58(),
      "Wrong reward mint"
    );
    assert(
      data.initialSupply.toNumber() ===
        initialSupply.toNumber() * 10 ** rewardMint.decimal,
      "Wrong initial supply"
    );
    assert(
      data.initialSupplyForCreator.toNumber() ===
        initialSupply.toNumber() * 10 ** rewardMint.decimal * 0.05,
      "Wrong initial supply for creator" + data.initialSupplyForCreator
    );
    assert(
      data.liquidityPoolSupply.toNumber() ===
        liquidityPoolSupply.toNumber() * 10 ** rewardMint.decimal,
      "Wrong liquidity pool supply"
    );
    assert(data.vestingPeriod === vestingPeriod, "Wrong vesting period");
    assert(data.vestingStartedAt === null, "Vesting should not have started");
  });

  step("Buy presale", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const quoteMint = NATIVE_MINT;
    const ixs = [];
    const poolQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        poolId,
        true,
        program.provider.connection
      );
    const payerQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        toWeb3JsPublicKey(signer.publicKey),
        true,
        program.provider.connection
      );
    const feeCollectorQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        new PublicKey("73hCTYpoZNdFiwbh2PrW99ykAyNcQVfUwPMUhu9ogNTg"),
        true,
        program.provider.connection
      );

    const amount = 0.2 * LAMPORTS_PER_SOL;
    if (quoteMint === NATIVE_MINT) {
      ixs.push(
        SystemProgram.transfer({
          fromPubkey: toWeb3JsPublicKey(signer.publicKey),
          toPubkey: payerQuoteMintTokenAccount,
          lamports: Math.round(amount * 1.1),
        })
      );
      ixs.push(createSyncNativeInstruction(payerQuoteMintTokenAccount));
    }
    try {
      ixs.push(
        await program.methods
          .buyPresale(new BN(amount))
          .accounts({
            quoteMint: quoteMint,
            purchaseAuthorisationRecord: null,
            poolQuoteMintTokenAccount: poolQuoteMintTokenAccount,
            payerQuoteMintTokenAccount: payerQuoteMintTokenAccount,
            feeCollectorQuoteMintTokenAccount:
              feeCollectorQuoteMintTokenAccount,
            program: program.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
            pool: poolId,
            nft: nftA.mintAddress,
            payer: signer.publicKey,
          })
          .signers([toWeb3JsKeypair(signer)])
          .instruction()
      );
      const tx = new VersionedTransaction(
        new TransactionMessage({
          instructions: ixs,
          recentBlockhash: (
            await program.provider.connection.getLatestBlockhash()
          ).blockhash,
          payerKey: toWeb3JsPublicKey(signer.publicKey),
        }).compileToV0Message()
      );
      tx.sign([toWeb3JsKeypair(signer)]);
      const txId = await program.provider.connection.sendTransaction(tx);
      await program.provider.connection.confirmTransaction(txId);
    } catch (e) {
      console.log(e);
    }
    const receipt = await program.account.purchaseReceipt.fetch(
      purchaseReceipt
    );
    assert(
      receipt.amount.toString() === amount.toString(),
      "Amount is not equal"
    );
    assert(receipt.lpClaimed.toNumber() === 0, "Claim should not have started");
    assert(
      receipt.originalMint.toBase58() === nftA.mintAddress.toBase58(),
      "Nft registered is wrong"
    );
    const pool = await program.account.pool.fetch(poolId);
    assert(
      pool.liquidityCollected.toNumber() === amount,
      "Pool Liquidity not equal"
    );
    const poolWsolAmount = await getAccount(
      program.provider.connection,
      poolQuoteMintTokenAccount
    );
    assert(Number(poolWsolAmount.amount) === amount, "WSOL amount not equal");
  });

  step("Buy presale exceed presale target", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const ixs = [];
    const quoteMint = NATIVE_MINT;
    const poolQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        poolId,
        true,
        program.provider.connection
      );
    const payerQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        toWeb3JsPublicKey(signer.publicKey),
        true,
        program.provider.connection
      );
    const feeCollectorQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        new PublicKey("73hCTYpoZNdFiwbh2PrW99ykAyNcQVfUwPMUhu9ogNTg"),
        true,
        program.provider.connection
      );

    const amount = 0.5 * LAMPORTS_PER_SOL;
    if (quoteMint === NATIVE_MINT) {
      ixs.push(
        SystemProgram.transfer({
          fromPubkey: toWeb3JsPublicKey(signer.publicKey),
          toPubkey: payerQuoteMintTokenAccount,
          lamports: Math.round(amount * 1.1),
        })
      );
      ixs.push(createSyncNativeInstruction(payerQuoteMintTokenAccount));
    }
    let failed = false;
    try {
      ixs.push(
        await program.methods
          .buyPresale(new BN(amount))
          .accounts({
            quoteMint: quoteMint,
            purchaseAuthorisationRecord: null,
            poolQuoteMintTokenAccount: poolQuoteMintTokenAccount,
            payerQuoteMintTokenAccount: payerQuoteMintTokenAccount,
            feeCollectorQuoteMintTokenAccount:
              feeCollectorQuoteMintTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            pool: poolId,
            nft: nftA.mintAddress,
            payer: signer.publicKey,
          })
          .signers([toWeb3JsKeypair(signer)])
          .instruction()
      );
      const tx = new VersionedTransaction(
        new TransactionMessage({
          instructions: ixs,
          recentBlockhash: (
            await program.provider.connection.getLatestBlockhash()
          ).blockhash,
          payerKey: toWeb3JsPublicKey(signer.publicKey),
        }).compileToV0Message()
      );
      tx.sign([toWeb3JsKeypair(signer)]);
      const txId = await program.provider.connection.sendTransaction(tx);
      await program.provider.connection.confirmTransaction(txId);
    } catch (e) {
      failed = true;
      console.log(e);
    }
    assert(failed, "Should fail because you do not own the nft");
    const receipt = await program.account.purchaseReceipt.fetch(
      purchaseReceipt
    );
    assert(
      receipt.amount.toNumber() === 0.2 * LAMPORTS_PER_SOL,
      "Amount is not equal"
    );
  });

  step("Buy presale without owning nft", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const quoteMint = NATIVE_MINT;
    const randomPayer = Keypair.generate();
    const ix = SystemProgram.transfer({
      fromPubkey: toWeb3JsPublicKey(signer.publicKey),
      toPubkey: randomPayer.publicKey,
      lamports: 0.4 * LAMPORTS_PER_SOL,
    });
    await sendAndConfirmTransaction(
      program.provider.connection,
      new Transaction().add(ix),
      [toWeb3JsKeypair(signer)]
    );

    const ixs = [];
    const poolQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        randomPayer.publicKey,
        quoteMint,
        poolId,
        true,
        program.provider.connection
      );
    const payerQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        randomPayer.publicKey,
        quoteMint,
        randomPayer.publicKey,
        true,
        program.provider.connection
      );
    const feeCollectorQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        randomPayer.publicKey,
        quoteMint,
        new PublicKey("73hCTYpoZNdFiwbh2PrW99ykAyNcQVfUwPMUhu9ogNTg"),
        true,
        program.provider.connection
      );
    const amount = 0.3 * LAMPORTS_PER_SOL;
    if (quoteMint === NATIVE_MINT) {
      ixs.push(
        SystemProgram.transfer({
          fromPubkey: randomPayer.publicKey,
          toPubkey: payerQuoteMintTokenAccount,
          lamports: Math.round(amount * 1.1),
        })
      );
      ixs.push(createSyncNativeInstruction(payerQuoteMintTokenAccount));
    }

    let failed = false;
    try {
      ixs.push(
        await program.methods
          .buyPresale(new BN(amount))
          .accounts({
            quoteMint: quoteMint,
            purchaseAuthorisationRecord: null,
            poolQuoteMintTokenAccount: poolQuoteMintTokenAccount,
            payerQuoteMintTokenAccount: payerQuoteMintTokenAccount,
            feeCollectorQuoteMintTokenAccount:
              feeCollectorQuoteMintTokenAccount,
            tokenProgram: TOKEN_PROGRAM_ID,
            pool: poolId,
            nft: nftA.mintAddress,
            payer: randomPayer.publicKey,
          })
          .signers([randomPayer])
          .instruction()
      );
      const tx = new VersionedTransaction(
        new TransactionMessage({
          instructions: ixs,
          recentBlockhash: (
            await program.provider.connection.getLatestBlockhash()
          ).blockhash,
          payerKey: randomPayer.publicKey,
        }).compileToV0Message()
      );
      tx.sign([randomPayer]);
      const txId = await program.provider.connection.sendTransaction(tx);
      await program.provider.connection.confirmTransaction(txId);
    } catch (e) {
      failed = true;
      console.log(e);
    }
    assert(!failed, "Should not fail");
    const receipt = await program.account.purchaseReceipt.fetch(
      purchaseReceipt
    );
    assert(
      receipt.amount.toNumber() === 0.5 * LAMPORTS_PER_SOL,
      "Amount is not equal"
    );
  });
  step("Buy presale exceed total cap", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const ixs = [];
    const quoteMint = NATIVE_MINT;
    const poolQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        poolId,
        true,
        program.provider.connection
      );
    const payerQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        toWeb3JsPublicKey(signer.publicKey),
        true,
        program.provider.connection
      );
    const feeCollectorQuoteMintTokenAccount =
      await getOrCreateAssociatedTokenAccountInstruction(
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        quoteMint,
        new PublicKey("73hCTYpoZNdFiwbh2PrW99ykAyNcQVfUwPMUhu9ogNTg"),
        true,
        program.provider.connection
      );

    const amount = 0.6 * LAMPORTS_PER_SOL;
    if (quoteMint === NATIVE_MINT) {
      ixs.push(
        SystemProgram.transfer({
          fromPubkey: toWeb3JsPublicKey(signer.publicKey),
          toPubkey: payerQuoteMintTokenAccount,
          lamports: Math.round(amount * 1.1),
        })
      );
      ixs.push(createSyncNativeInstruction(payerQuoteMintTokenAccount));
    }

    let failed = false;
    try {
      ixs.push(
        await program.methods
          .buyPresale(new BN(amount))
          .accounts({
            quoteMint: quoteMint,
            purchaseAuthorisationRecord: null,
            poolQuoteMintTokenAccount: poolQuoteMintTokenAccount,
            payerQuoteMintTokenAccount: payerQuoteMintTokenAccount,
            feeCollectorQuoteMintTokenAccount:
              feeCollectorQuoteMintTokenAccount,
            pool: poolId,
            tokenProgram: TOKEN_PROGRAM_ID,
            nft: nftA.mintAddress,
            payer: signer.publicKey,
          })
          .signers([toWeb3JsKeypair(signer)])
          .instruction()
      );
      const tx = new VersionedTransaction(
        new TransactionMessage({
          instructions: ixs,
          recentBlockhash: (
            await program.provider.connection.getLatestBlockhash()
          ).blockhash,
          payerKey: toWeb3JsPublicKey(signer.publicKey),
        }).compileToV0Message()
      );
      tx.sign([toWeb3JsKeypair(signer)]);
      const txId = await program.provider.connection.sendTransaction(tx);
      await program.provider.connection.confirmTransaction(txId);
    } catch (e) {
      failed = true;
      console.log(e);
    }
    assert(failed, "Should fail because amount exceeded capped set by creator");
    const receipt = await program.account.purchaseReceipt.fetch(
      purchaseReceipt
    );
    assert(
      receipt.amount.toNumber() === 0.5 * LAMPORTS_PER_SOL,
      "Amount is not equal"
    );
  });
  step("Random Person Create Market using your token", async () => {
    try {
      const randomPayer = Keypair.generate();
      const ix = SystemProgram.transfer({
        fromPubkey: toWeb3JsPublicKey(signer.publicKey),
        toPubkey: randomPayer.publicKey,
        lamports: 3 * LAMPORTS_PER_SOL,
      });
      await sendAndConfirmTransaction(
        program.provider.connection,
        new Transaction().add(ix),
        [toWeb3JsKeypair(signer)]
      );
      const { innerTransactions, address } =
        await MarketV2.makeCreateMarketInstructionSimple({
          connection: program.provider.connection,
          wallet: randomPayer.publicKey,
          baseInfo: {
            mint: rewardMint.mint,
            decimals: rewardMint.decimal,
          },
          quoteInfo: {
            mint: new PublicKey(WSOL.mint),
            decimals: WSOL.decimals,
          },
          lotSize: 1,
          tickSize: 0.000001,
          dexProgramId: DEVNET_PROGRAM_ID.OPENBOOK_MARKET,
          makeTxVersion: TxVersion.LEGACY,
        });
      const txs = await buildSimpleTransaction({
        connection: program.provider.connection,
        makeTxVersion: TxVersion.LEGACY,
        payer: randomPayer.publicKey,
        innerTransactions,
      });
      for (let tx of txs) {
        (tx as Transaction).sign(randomPayer);
        const rawTransaction = tx.serialize();
        const txid: TransactionSignature =
          await program.provider.connection.sendRawTransaction(rawTransaction, {
            skipPreflight: true,
          });
        const confirmation =
          await program.provider.connection.confirmTransaction(txid);
        if (confirmation.value.err) {
          console.error(JSON.stringify(confirmation.value.err.valueOf()));
          throw Error("Insufficient SOL");
        }
      }
      const accountInfo = await program.provider.connection.getAccountInfo(
        randomPayer.publicKey
      );
      console.log(
        (3 * LAMPORTS_PER_SOL - accountInfo.lamports) / LAMPORTS_PER_SOL
      );
      ammInfo = address;
    } catch (e) {
      console.log(e);
    }
  });

  xstep("Create Market for AMM ", async () => {
    try {
      const { innerTransactions, address } =
        await MarketV2.makeCreateMarketInstructionSimple({
          connection: program.provider.connection,
          wallet: toWeb3JsPublicKey(signer.publicKey),
          baseInfo: {
            mint: rewardMint.mint,
            decimals: rewardMint.decimal,
          },
          quoteInfo: {
            mint: new PublicKey(WSOL.mint),
            decimals: WSOL.decimals,
          },
          lotSize: 1,
          tickSize: 0.000001,
          dexProgramId: DEVNET_PROGRAM_ID.OPENBOOK_MARKET,
          makeTxVersion: TxVersion.LEGACY,
        });
      const txs = await buildSimpleTransaction({
        connection: program.provider.connection,
        makeTxVersion: TxVersion.LEGACY,
        payer: toWeb3JsPublicKey(signer.publicKey),
        innerTransactions,
      });
      for (let tx of txs) {
        (tx as Transaction).sign(toWeb3JsKeypair(signer));
        const rawTransaction = tx.serialize();
        const txid: TransactionSignature =
          await program.provider.connection.sendRawTransaction(rawTransaction, {
            skipPreflight: true,
          });
        const confirmation =
          await program.provider.connection.confirmTransaction(txid);
        if (confirmation.value.err) {
          console.error(JSON.stringify(confirmation.value.err.valueOf()));
          throw Error("Insufficient SOL");
        }
      }
    } catch (e) {
      console.log(e);
    }
  });

  step("Launch Token for AMM", async () => {
    setTimeout(() => {}, 30000);
    const poolInfo = Liquidity.getAssociatedPoolKeys({
      version: 4,
      marketVersion: 3,
      marketId: ammInfo.marketId,
      baseMint: ammInfo.baseMint,
      quoteMint: ammInfo.quoteMint,
      baseDecimals: 6,
      quoteDecimals: WSOL.decimals,
      programId: DEVNET_PROGRAM_ID.AmmV4,
      marketProgramId: DEVNET_PROGRAM_ID.OPENBOOK_MARKET,
    });

    const poolTokenLp = getAssociatedTokenAddressSync(
      poolInfo.lpMint,
      poolId,
      true
    );
    const userTokenLp = getAssociatedTokenAddressSync(
      poolInfo.lpMint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const remainingAccounts = [
      { pubkey: poolInfo.lpMint, isSigner: false, isWritable: true },
      { pubkey: userTokenLp, isSigner: false, isWritable: true },
      { pubkey: poolTokenLp, isSigner: false, isWritable: true },
      { pubkey: poolInfo.id, isSigner: false, isWritable: true },
      { pubkey: poolInfo.authority, isSigner: false, isWritable: false },
      {
        pubkey: poolInfo.openOrders,
        isSigner: false,
        isWritable: true,
      },
      { pubkey: poolInfo.baseVault, isSigner: false, isWritable: true },
      { pubkey: poolInfo.quoteVault, isSigner: false, isWritable: true },
      { pubkey: poolInfo.targetOrders, isSigner: false, isWritable: true },
      { pubkey: poolInfo.configId, isSigner: false, isWritable: false },
      {
        pubkey: new PublicKey("3XMrhbv989VxAMi3DErLV9eJht1pHppW5LbKxe9fkEFR"),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: poolInfo.marketProgramId, isSigner: false, isWritable: false },
      { pubkey: poolInfo.marketId, isSigner: false, isWritable: false },
    ];

    const userTokenCoin = getAssociatedTokenAddressSync(
      poolInfo.baseMint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const userTokenPc = getAssociatedTokenAddressSync(
      poolInfo.quoteMint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const poolTokenCoin = getAssociatedTokenAddressSync(
      poolInfo.baseMint,
      poolId,
      true
    );
    const poolTokenPc = getAssociatedTokenAddressSync(
      poolInfo.quoteMint,
      poolId,
      true
    );
    const ixs = [];
    ixs.push(
      await program.methods
        .launchTokenAmm(poolInfo.nonce, new BN(Date.now()))
        .accounts({
          pool: poolId,
          userWallet: signer.publicKey,
          userTokenCoin: userTokenCoin,
          userTokenPc: userTokenPc,
          poolTokenPc: poolTokenPc,
          poolTokenCoin: poolTokenCoin,
          tokenProgram: TOKEN_PROGRAM_ID,
          ammCoinMint: poolInfo.baseMint,
          ammPcMint: poolInfo.quoteMint,
        })
        .signers([toWeb3JsKeypair(signer)])
        .remainingAccounts(remainingAccounts)
        .instruction()
    );

    const [microLamports, units, recentBlockhash] = await Promise.all([
      100,
      getSimulationUnits(
        program.provider.connection,
        ixs,
        toWeb3JsPublicKey(signer.publicKey),
        []
      ),
      program.provider.connection.getLatestBlockhash(),
    ]);
    ixs.unshift(ComputeBudgetProgram.setComputeUnitPrice({ microLamports }));
    if (units) {
      // probably should add some margin of error to units
      console.log(units);
      ixs.unshift(
        ComputeBudgetProgram.setComputeUnitLimit({ units: units * 1.1 })
      );
    }

    const tx = new VersionedTransaction(
      new TransactionMessage({
        instructions: ixs,
        recentBlockhash: recentBlockhash.blockhash,
        payerKey: toWeb3JsPublicKey(signer.publicKey),
      }).compileToV0Message()
    );
    tx.sign([toWeb3JsKeypair(signer)]);
    let txId;
    try {
      txId = await program.provider.connection.sendTransaction(tx);
      const confirmation = await program.provider.connection.confirmTransaction(
        txId
      );
      if (confirmation.value.err) {
        txId = null;
        console.error(JSON.stringify(confirmation.value.err.valueOf()));
      } else {
        console.log(txId);
      }
    } catch (e) {
      console.log(txId);
      txId = null;
      console.log(e.logs.length > 50 ? e.logs.slice(-50) : e);
    }
    assert(txId !== null, "Failed Transaction");
  });

  step("Check Claim Rewards", async () => {
    const poolData = await program.account.pool.fetch(poolId);

    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const purchaseReceiptLpTokenAccount = getAssociatedTokenAddressSync(
      poolData.lpMint,
      purchaseReceipt,
      true
    );
    const purchaseReceiptRewardTokenAccount = getAssociatedTokenAddressSync(
      poolData.mint,
      purchaseReceipt,
      true
    );
    const poolLpTokenAccount = getAssociatedTokenAddressSync(
      poolData.lpMint,
      poolId,
      true
    );
    const poolRewardTokenAccount = getAssociatedTokenAddressSync(
      poolData.mint,
      poolId,
      true
    );
    try {
      await program.methods
        .checkClaimEllgibility()
        .accounts({
          purchaseReceiptRewardTokenAccount: purchaseReceiptRewardTokenAccount,
          poolRewardTokenAccount: poolRewardTokenAccount,
          rewardMint: rewardMint.mint,
          purchaseReceiptLpTokenAccount: purchaseReceiptLpTokenAccount,
          poolLpTokenAccount: poolLpTokenAccount,
          purchaseReceipt: purchaseReceipt,
          lpMint: poolData.lpMint,
          pool: poolId,
          payer: toWeb3JsPublicKey(signer.publicKey),
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
    } catch (e) {
      console.log(e);
    }
  });

  step("Claim lp token", async () => {
    const poolData = await program.account.pool.fetch(poolId);

    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const purchaseReceiptLpTokenAccount = getAssociatedTokenAddressSync(
      poolData.lpMint,
      purchaseReceipt,
      true
    );
    const nftOwnerLpTokenAccount = getAssociatedTokenAddressSync(
      poolData.lpMint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const payerOriginalMintAta = getAssociatedTokenAddressSync(
      nftA.mintAddress,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const poolAuthorityLpTokenAccount = getAssociatedTokenAddressSync(
      poolData.lpMint,
      poolData.authority,
      true
    );
    try {
      const txId = await program.methods
        .withdrawLpTokens()
        .accounts({
          purchaseReceiptLpTokenAccount: purchaseReceiptLpTokenAccount,
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          lpMint: poolData.lpMint,
          nftOwner: signer.publicKey,
          nftOwnerNftTokenAccount: payerOriginalMintAta,
          nftOwnerLpTokenAccount: nftOwnerLpTokenAccount,
          payer: signer.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
      const confirmation = await program.provider.connection.confirmTransaction(
        txId
      );
      if (confirmation.value.err) {
        console.log("Error");
      }
    } catch (e) {
      console.log(e);
    }
    const data = await program.account.purchaseReceipt.fetch(purchaseReceipt);
    console.log(data);
  });
});
