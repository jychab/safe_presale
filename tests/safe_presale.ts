import { SafePresale } from "../target/types/safe_presale";
import {
  MPL_TOKEN_METADATA_PROGRAM_ID,
  findMasterEditionPda,
  findMetadataPda,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";
import { keypairIdentity, publicKey } from "@metaplex-foundation/umi";

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
  toWeb3JsKeypair,
  toWeb3JsPublicKey,
} from "@metaplex-foundation/umi-web3js-adapters";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount,
  transfer,
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
import { getSimulationUnits } from "./utils";

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

  let identifierId: PublicKey;
  let identifier;
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
      //   let madLadCollection = generateSigner(umi);
      //   await createNft(umi, {
      //     mint: madLadCollection,
      //     name: "Mad Lad Collection",
      //     uri: "https://example.com/my-collection.json",
      //     sellerFeeBasisPoints: percentAmount(5.5), // 5.5%
      //     creators: [{ address: signer.publicKey, verified: true, share: 100 }],
      //     isCollection: true,
      //   }).sendAndConfirm(umi);
      collection = {
        mintAddress: new PublicKey(
          "3gb8ETqmiobYiT4k8dpfRo9RemhkfdF82tZWzaXcZkov"
        ),
        masterEditionAddress: toWeb3JsPublicKey(
          findMasterEditionPda(umi, {
            mint: publicKey("3gb8ETqmiobYiT4k8dpfRo9RemhkfdF82tZWzaXcZkov"),
          })[0]
        ),
        metadataAddress: toWeb3JsPublicKey(
          findMetadataPda(umi, {
            mint: publicKey("3gb8ETqmiobYiT4k8dpfRo9RemhkfdF82tZWzaXcZkov"),
          })[0]
        ),
      };
      //   let madlad1 = generateSigner(umi);
      //   await createNft(umi, {
      //     mint: madlad1,
      //     name: "MadLad 1",
      //     uri: "https://arweave.net/my-content-hash",
      //     sellerFeeBasisPoints: percentAmount(5.5), // 5.5%
      //     isMutable: true,
      //     collection: {
      //       key: madLadCollection.publicKey,
      //       verified: false,
      //     },
      //   }).sendAndConfirm(umi);
      nftA = {
        mintAddress: new PublicKey(
          "BbsGAmRWneqgdnk9NEpD48oBMPkziGrdeqN4FtRcRD94"
        ),
        masterEditionAddress: toWeb3JsPublicKey(
          findMasterEditionPda(umi, {
            mint: publicKey("BbsGAmRWneqgdnk9NEpD48oBMPkziGrdeqN4FtRcRD94"),
          })[0]
        ),
        metadataAddress: toWeb3JsPublicKey(
          findMetadataPda(umi, {
            mint: publicKey("BbsGAmRWneqgdnk9NEpD48oBMPkziGrdeqN4FtRcRD94"),
          })[0]
        ),
      };
      //   await verifyCollectionV1(umi, {
      //     metadata: findMetadataPda(umi, { mint: madlad1.publicKey }),
      //     collectionMint: madLadCollection.publicKey,
      //     authority: umi.payer,
      //   }).sendAndConfirm(umi);
    }
  );

  step("Initialize an identifier if required", async () => {
    [identifierId] = PublicKey.findProgramAddressSync(
      [Buffer.from("identifier")],
      program.programId
    );
    const identifierData = await program.account.identifier.fetchNullable(
      identifierId
    );

    identifier =
      identifierData !== null ? identifierData.count : new anchor.BN(1);

    if (!identifierData) {
      try {
        await program.methods
          .initIdentifier()
          .accounts({
            identifier: identifierId,
            payer: signer.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([toWeb3JsKeypair(signer)])
          .rpc();
      } catch (e) {
        console.log(e);
      }
    }
  });

  step("Initialize a pool", async () => {
    [poolId] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), identifier.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    const [rewardMintKey] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), identifier.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    rewardMint = {
      mint: rewardMintKey,
      name: "XYZ",
      symbol: "Fock",
      decimal: 5,
      uri: "https://www.madlads.com/mad_lads_logo.svg",
    };
    const totalSupply = new BN(1000000000);
    const vestedSupply = new BN(500000000);
    const vestingPeriod = 3 * 24 * 60 * 60; //3days in seconds
    const maxPresaleTime = 1; // in seconds

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
          name: rewardMint.name,
          symbol: rewardMint.symbol,
          decimals: rewardMint.decimal,
          uri: rewardMint.uri,
          maxPresaleTime: maxPresaleTime,
          presaleTarget: new BN(10),
          creatorFeeBasisPoints: 500,
          delegate: null,
          vestingPeriod: vestingPeriod,
          vestedSupply: vestedSupply,
          totalSupply: totalSupply,
        })
        .accounts({
          payer: signer.publicKey,
          pool: poolId,
          rewardMint: rewardMint.mint,
          identifier: identifierId,
          rewardMintMetadata: rewardMint_metadata,
          poolRewardMintAta: poolAndMintRewardAta,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          mplTokenProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
      const data = await program.account.pool.fetch(poolId);
      assert(data.launched === false, "Not allowed for purchase");
      assert(
        data.authority.toBase58() === signer.publicKey.toString(),
        "Wrong authority"
      );
      assert(
        data.identifier.toNumber() === (identifier as BN).toNumber(),
        "Wrong identifier"
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
        data.totalSupply.toNumber() ===
          totalSupply.toNumber() * 10 ** rewardMint.decimal,
        "Wrong total supply"
      );
      assert(
        data.vestedSupply.toNumber() ===
          vestedSupply.toNumber() * 10 ** rewardMint.decimal,
        "Wrong vested supply"
      );
      assert(data.vestingPeriod === vestingPeriod, "Wrong vesting period");
      assert(data.vestingStartedAt === null, "Vesting should not have started");
      assert(data.vestingPeriodEnd === null, "Vesting should not have ended");
    } catch (e) {
      console.log(e);
    }
  });

  step("Buy presale", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const poolAndWSOLATA = getAssociatedTokenAddressSync(
      new PublicKey(WSOL.mint),
      poolId,
      true
    );

    const amount = new BN(0.1 * LAMPORTS_PER_SOL);
    try {
      await program.methods
        .buyPresale(amount)
        .accounts({
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          wsolMint: new PublicKey(WSOL.mint),
          poolWsolTokenAccount: poolAndWSOLATA,
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          nft: nftA.mintAddress,
          payer: signer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
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
    assert(
      receipt.mintClaimed.toNumber() === 0,
      "Claim should not have started"
    );
    assert(
      receipt.originalMint.toBase58() === nftA.mintAddress.toBase58(),
      "Nft registered is wrong"
    );
    const pool = await program.account.pool.fetch(poolId);
    assert(
      pool.liquidityCollected.toNumber() === amount.toNumber(),
      "Pool Liquidity not equal"
    );
    const poolWsolAmount = await getAccount(
      program.provider.connection,
      poolAndWSOLATA
    );
    assert(
      Number(poolWsolAmount.amount) === amount.toNumber(),
      "WSOL amount not equal"
    );
  });
  step("Buy presale without owning nft", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const poolAndWSOLATA = getAssociatedTokenAddressSync(
      new PublicKey(WSOL.mint),
      poolId,
      true
    );
    const randomPayer = Keypair.generate();
    const ix = SystemProgram.transfer({
      fromPubkey: toWeb3JsPublicKey(signer.publicKey),
      toPubkey: randomPayer.publicKey,
      lamports: 0.1 * LAMPORTS_PER_SOL,
    });
    await sendAndConfirmTransaction(
      program.provider.connection,
      new Transaction().add(ix),
      [toWeb3JsKeypair(signer)]
    );
    const amount = new BN(0.1 * LAMPORTS_PER_SOL);
    try {
      await program.methods
        .buyPresale(amount)
        .accounts({
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          wsolMint: new PublicKey(WSOL.mint),
          poolWsolTokenAccount: poolAndWSOLATA,
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          nft: nftA.mintAddress,
          payer: randomPayer.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([randomPayer])
        .rpc();
    } catch (e) {
      console.log(e);
    }
    const receipt = await program.account.purchaseReceipt.fetch(
      purchaseReceipt
    );
    assert(
      receipt.amount.toNumber() === amount.toNumber() * 2,
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

    const remainingAccounts = [
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
    const userTokenLp = getAssociatedTokenAddressSync(
      poolInfo.lpMint,
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
    const poolTokenLp = getAssociatedTokenAddressSync(
      poolInfo.lpMint,
      poolId,
      true
    );
    const ixs = [];
    ixs.push(
      await program.methods
        .launchTokenAmm(poolInfo.nonce, new BN(Date.now()))
        .accounts({
          pool: poolId,
          poolAuthority: signer.publicKey,
          userWallet: signer.publicKey,
          userTokenCoin: userTokenCoin,
          userTokenPc: userTokenPc,
          userTokenLp: userTokenLp,
          poolTokenPc: poolTokenPc,
          poolTokenCoin: poolTokenCoin,
          poolTokenLp: poolTokenLp,
          rent: RENT_PROGRAM_ID,
          systemProgram: SYSTEM_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          ammCoinMint: poolInfo.baseMint,
          ammPcMint: poolInfo.quoteMint,
          ammLpMint: poolInfo.lpMint,
          raydiumAmmProgram: DEVNET_PROGRAM_ID.AmmV4,
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
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    try {
      await program.methods
        .checkClaimEllgibility()
        .accounts({
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          payer: toWeb3JsPublicKey(signer.publicKey),
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
    } catch (e) {
      console.log(e);
    }
  });

  step("Claim rewards", async () => {
    [purchaseReceipt] = PublicKey.findProgramAddressSync(
      [Buffer.from("receipt"), poolId.toBuffer(), nftA.mintAddress.toBuffer()],
      program.programId
    );
    const payerOriginalMintAta = getAssociatedTokenAddressSync(
      nftA.mintAddress,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const payerRewardMintTokenAccount = getAssociatedTokenAddressSync(
      rewardMint.mint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    try {
      const txId = await program.methods
        .claimRewards()
        .accounts({
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          nft: nftA.mintAddress,
          nftOwner: signer.publicKey,
          nftOwnerNftTokenAccount: payerOriginalMintAta,
          rewardMint: rewardMint.mint,
          nftOwnerRewardMintTokenAccount: payerRewardMintTokenAccount,
          payer: signer.publicKey,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([toWeb3JsKeypair(signer)])
        .rpc();
      const confirmation = await program.provider.connection.confirmTransaction(
        txId
      );
      if (confirmation.value.err) {
        console.log("Error");
      }
      const data = await program.account.purchaseReceipt.fetch(purchaseReceipt);
      console.log(data);
    } catch (e) {
      console.log(e);
    }
  });
});
