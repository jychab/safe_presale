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
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  fromWeb3JsPublicKey,
  toWeb3JsKeypair,
  toWeb3JsPublicKey,
} from "@metaplex-foundation/umi-web3js-adapters";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  getAccount,
  unpackMint,
} from "@solana/spl-token";
import { step, xstep } from "mocha-steps";
import {
  Clmm,
  ClmmConfigInfo,
  ClmmPoolInfo,
  DEVNET_PROGRAM_ID,
  Fraction,
  METADATA_PROGRAM_ID,
  ObservationInfoLayout,
  POOL_VAULT_SEED,
  POSITION_SEED,
  PoolUtils,
  RENT_PROGRAM_ID,
  ReturnTypeFetchMultipleMintInfos,
  SYSTEM_PROGRAM_ID,
  SqrtPriceMath,
  TICK_ARRAY_SEED,
  TickUtils,
  TxVersion,
  WSOL,
  buildSimpleTransaction,
  generatePubKey,
  getMultipleAccountsInfo,
  getPdaAmmConfigId,
  getPdaExBitmapAccount,
  getPdaOperationAccount,
  getPdaPersonalPositionAddress,
  getPdaPoolId,
  getPdaPoolVaultId,
  getPdaProtocolPositionAddress,
  getPdaTickArrayAddress,
} from "@raydium-io/raydium-sdk";
import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { assert } from "chai";
import Decimal from "decimal.js";
import { SPL_TOKEN_2022_PROGRAM_ID } from "../deps/mpl-token-metadata/clients/js/test/_setup";
import { IDL as ClmmIDL, CLMM } from "./clmm";

describe("XBadge", () => {
  // Configure the client to use the local cluster.
  // Use the RPC endpoint of your choice.
  anchor.setProvider(
    new anchor.AnchorProvider(
      new Connection("https://api.devnet.solana.com"),
      new anchor.Wallet(
        Keypair.fromSecretKey(
          Buffer.from([
            225, 66, 240, 160, 100, 176, 216, 156, 98, 248, 136, 34, 108, 179,
            97, 33, 245, 103, 165, 252, 153, 131, 20, 190, 60, 85, 11, 240, 176,
            184, 50, 183, 208, 37, 214, 8, 236, 36, 232, 48, 167, 48, 193, 156,
            104, 55, 81, 126, 209, 94, 147, 84, 22, 209, 65, 127, 206, 246, 2,
            145, 207, 168, 186, 29,
          ])
        )
      ),
      { commitment: "finalized" }
    )
  );

  //
  // Program APIs.
  //

  const program = anchor.workspace.XBadge as Program<SafePresale>;
  const clmmProgram = new Program<CLMM>(ClmmIDL, DEVNET_PROGRAM_ID.CLMM);
  const umi = createUmi(program.provider.connection.rpcEndpoint);
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
  let rewardMint: PublicKey;
  let purchaseReceipt: PublicKey;
  let clmmInfo: ClmmPoolInfo;

  //
  // NFTs. These are the two mad lads for the tests.
  //
  let nftA: {
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

    [rewardMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("mint"), poolId.toBuffer()],
      program.programId
    );

    const [rewardMint_metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID).toBuffer(),
        rewardMint.toBuffer(),
      ],
      toWeb3JsPublicKey(MPL_TOKEN_METADATA_PROGRAM_ID)
    );

    const poolAndMintRewardAta = getAssociatedTokenAddressSync(
      rewardMint,
      poolId,
      true
    );

    try {
      await program.methods
        .initPool({
          name: "Fock it.",
          symbol: "Fock",
          decimals: 9,
          uri: "https://www.madlads.com/mad_lads_logo.svg",
          requiresCollections: [collection.mintAddress],
          vestingPeriod: new BN(Date.now()),
          supplyForInitialLiquidity: new BN(500000000),
          totalSupply: new BN(1000000000),
        })
        .accounts({
          payer: signer.publicKey,
          pool: poolId,
          rewardMint: rewardMint,
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
      assert(data.isClosed === false);
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

    const amount = new BN(1000);
    try {
      await program.methods
        .buyPresale(amount)
        .accounts({
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          wsolMint: new PublicKey(WSOL.mint),
          poolWsolTokenAccount: poolAndWSOLATA,
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          originalMint: nftA.mintAddress,
          originalMintMetadata: nftA.metadataAddress,
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

    const pool = await program.account.pool.fetch(poolId);
    assert(
      pool.liquidityCollected.toString() === amount.toString(),
      "Pool Liquidity not equal"
    );

    const poolWsolAmount = await getAccount(
      program.provider.connection,
      poolAndWSOLATA
    );
    assert(
      poolWsolAmount.amount.toString() === amount.toString(),
      "WSOL amount not equal"
    );
  });

  step("Create Market", async () => {
    const config: ClmmConfigInfo = {
      id: getPdaAmmConfigId(DEVNET_PROGRAM_ID.CLMM, 3).publicKey,
      index: 3,
      protocolFeeRate: 120000, //raydium fees
      tradeFeeRate: 10000,
      tickSpacing: 120,
      description: "Best for exotic pairs",
      fundFeeRate: 40000,
      fundOwner: "FundHfY8oo8J9KYGyfXFFuQCHe7Z1VBNmsj84eMcdYs4", // raydium fees
    };

    const poolData = await program.account.pool.fetch(poolId);

    const initialPrice = new Decimal(
      new Fraction(
        poolData.liquidityCollected,
        poolData.supplyForInitialLiquidity.toNumber()
      ).toFixed(12)
    );
    const mint1 = {
      mint: rewardMint,
      decimals: 9,
      programId: TOKEN_PROGRAM_ID,
    };
    const mint2 = {
      mint: new PublicKey(WSOL.mint),
      decimals: WSOL.decimals,
      programId: TOKEN_PROGRAM_ID,
    };

    const [mintA, mintB, initPrice] = new BN(mint1.mint.toBuffer()).gt(
      new BN(mint2.mint.toBuffer())
    )
      ? [mint2, mint1, new Decimal(1).div(initialPrice)]
      : [mint1, mint2, initialPrice];
    const initialPriceX64 = SqrtPriceMath.priceToSqrtPriceX64(
      initPrice,
      mintA.decimals,
      mintB.decimals
    );
    const observationId = generatePubKey({
      fromPublicKey: toWeb3JsPublicKey(signer.publicKey),
      programId: DEVNET_PROGRAM_ID.CLMM,
    });
    const poolStateId = getPdaPoolId(
      DEVNET_PROGRAM_ID.CLMM,
      config.id,
      mintA.mint,
      mintB.mint
    ).publicKey;
    const mintAVault = getPdaPoolVaultId(
      DEVNET_PROGRAM_ID.CLMM,
      poolStateId,
      mintA.mint
    ).publicKey;
    const mintBVault = getPdaPoolVaultId(
      DEVNET_PROGRAM_ID.CLMM,
      poolStateId,
      mintB.mint
    ).publicKey;

    try {
      const ix = [];
      ix.push(
        SystemProgram.createAccountWithSeed({
          fromPubkey: toWeb3JsPublicKey(signer.publicKey),
          basePubkey: toWeb3JsPublicKey(signer.publicKey),
          seed: observationId.seed,
          newAccountPubkey: observationId.publicKey,
          lamports:
            await program.provider.connection.getMinimumBalanceForRentExemption(
              ObservationInfoLayout.span
            ),
          space: ObservationInfoLayout.span,
          programId: DEVNET_PROGRAM_ID.CLMM,
        })
      );
      ix.push(
        await clmmProgram.methods
          .createPool(initialPriceX64, new BN(Date.now()))
          .accounts({
            poolCreator: toWeb3JsPublicKey(signer.publicKey),
            ammConfig: config.id,
            poolState: poolStateId,
            tokenMint0: mintA.mint,
            tokenMint1: mintB.mint,
            tokenVault0: mintAVault,
            tokenVault1: mintBVault,
            observationState: observationId.publicKey,
            tickArrayBitmap: getPdaExBitmapAccount(
              DEVNET_PROGRAM_ID.CLMM,
              poolStateId
            ).publicKey,
            tokenProgram0: mintA.programId,
            tokenProgram1: mintB.programId,
            systemProgram: SYSTEM_PROGRAM_ID,
            rent: RENT_PROGRAM_ID,
          })
          .signers([toWeb3JsKeypair(signer)])
          .instruction()
      );
      const blockhash = await program.provider.connection.getLatestBlockhash();
      const messageV0 = new TransactionMessage({
        payerKey: toWeb3JsPublicKey(signer.publicKey),
        recentBlockhash: blockhash.blockhash,
        instructions: ix,
      }).compileToV0Message();
      const transaction = new VersionedTransaction(messageV0);
      transaction.sign([toWeb3JsKeypair(signer)]);
      const txid = await program.provider.connection.sendTransaction(
        transaction
      );
      const confirmation = await program.provider.connection.confirmTransaction(
        {
          signature: txid,
          blockhash: blockhash.blockhash,
          lastValidBlockHeight: blockhash.lastValidBlockHeight,
        }
      );
      if (confirmation.value.err) {
        console.log("Transaction has error");
      }
    } catch (e) {
      console.log(e);
    }
    clmmInfo = Clmm.makeMockPoolInfo({
      ammConfig: config,
      mint1: {
        mint: rewardMint,
        decimals: 9,
        programId: TOKEN_PROGRAM_ID,
      },
      mint2: {
        mint: new PublicKey(WSOL.mint),
        decimals: WSOL.decimals,
        programId: TOKEN_PROGRAM_ID,
      },
      owner: toWeb3JsPublicKey(signer.publicKey),
      programId: DEVNET_PROGRAM_ID.CLMM,
      createPoolInstructionSimpleAddress: {
        observationId: observationId.publicKey,
        poolId: poolStateId,
        mintA: mintA.mint,
        mintB: mintB.mint,
        mintAVault: mintAVault,
        mintBVault: mintBVault,
        mintProgramIdA: mintA.programId,
        mintProgramIdB: mintB.programId,
      },
      initialPrice: initialPrice,
      startTime: new BN(Date.now()),
    });
  });

  step("Launch Token", async () => {
    const poolData = await program.account.pool.fetch(poolId);
    const remainingAmount = poolData.totalSupply.sub(
      poolData.supplyForInitialLiquidity
    );

    const lowerPriceAndTick = Clmm.getPriceAndTick({
      poolInfo: clmmInfo,
      price: new Decimal(
        Math.max(clmmInfo.currentPrice.mul(0.5).toNumber(), 1 / 10 ** 6)
      ),
      baseIn: !clmmInfo.mintA.mint.equals(rewardMint),
    });
    const lowerTickArrayStartIndex = TickUtils.getTickArrayStartIndexByTick(
      lowerPriceAndTick.tick,
      clmmInfo.ammConfig.tickSpacing
    );

    const upperPriceAndTick = Clmm.getPriceAndTick({
      poolInfo: clmmInfo,
      price: new Decimal(
        Math.max(clmmInfo.currentPrice.mul(1.5).toNumber(), 1 / 10 ** 6)
      ),
      baseIn: !clmmInfo.mintA.mint.equals(rewardMint),
    });
    const upperTickArrayStartIndex = TickUtils.getTickArrayStartIndexByTick(
      upperPriceAndTick.tick,
      clmmInfo.ammConfig.tickSpacing
    );
    const mintInfos = await getMultipleAccountsInfo(
      program.provider.connection,
      [clmmInfo.mintA.mint, clmmInfo.mintB.mint]
    );
    const token2022Infos: ReturnTypeFetchMultipleMintInfos = {
      [clmmInfo.mintA.mint.toBase58()]: {
        ...unpackMint(clmmInfo.mintA.mint, mintInfos[0]),
        feeConfig: undefined,
      },
      [clmmInfo.mintB.mint.toBase58()]: {
        ...unpackMint(clmmInfo.mintB.mint, mintInfos[1]),
        feeConfig: undefined,
      },
    };
    const isFocus1 = true;
    const isCoin1Base = clmmInfo.mintA.mint.equals(rewardMint);
    const isPairPoolDirectionEq =
      (isFocus1 && isCoin1Base) || (!isCoin1Base && !isFocus1);
    const amountRequired = Clmm.getLiquidityAmountOutFromAmountIn({
      poolInfo: clmmInfo,
      slippage: 0,
      inputA: isPairPoolDirectionEq,
      tickUpper: Math.max(lowerPriceAndTick.tick, upperPriceAndTick.tick),
      tickLower: Math.min(lowerPriceAndTick.tick, upperPriceAndTick.tick),
      amount: new BN(1000),
      add: false,
      epochInfo: await program.provider.connection.getEpochInfo(),
      token2022Infos: token2022Infos,
      amountHasFee: true,
    });
    const coin1CalcedResult = isCoin1Base
      ? amountRequired.amountA
      : amountRequired.amountB;
    const coin2CalcedResult = isCoin1Base
      ? amountRequired.amountB
      : amountRequired.amountA;
    const token0_amount = coin1CalcedResult.amount;
    const token1_amount = coin2CalcedResult.amount;
    const position_nft_mint = Keypair.generate();
    const [position_nft_mint_metadata] = findMetadataPda(umi, {
      mint: fromWeb3JsPublicKey(position_nft_mint.publicKey),
    });
    const poolAndPositionNftAta = getAssociatedTokenAddressSync(
      position_nft_mint.publicKey,
      poolId,
      true
    );
    const [vaultAndToken0Ata] = PublicKey.findProgramAddressSync(
      [POOL_VAULT_SEED, clmmInfo.id.toBuffer(), clmmInfo.mintA.mint.toBuffer()],
      DEVNET_PROGRAM_ID.CLMM
    );
    const payerAndToken0Ata = getAssociatedTokenAddressSync(
      clmmInfo.mintA.mint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const poolAndToken0ATA = getAssociatedTokenAddressSync(
      clmmInfo.mintA.mint,
      poolId,
      true
    );
    const [vaultAndToken1ATA] = PublicKey.findProgramAddressSync(
      [POOL_VAULT_SEED, clmmInfo.id.toBuffer(), clmmInfo.mintB.mint.toBuffer()],
      DEVNET_PROGRAM_ID.CLMM
    );
    const payerAndToken1Ata = getAssociatedTokenAddressSync(
      clmmInfo.mintB.mint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    const poolAndToken1ATA = getAssociatedTokenAddressSync(
      clmmInfo.mintB.mint,
      poolId,
      true
    );
    const protocol_position = getPdaProtocolPositionAddress(
      DEVNET_PROGRAM_ID.CLMM,
      clmmInfo.id,
      lowerPriceAndTick.tick,
      upperPriceAndTick.tick
    ).publicKey;
    const tick_array_lower = getPdaTickArrayAddress(
      DEVNET_PROGRAM_ID.CLMM,
      clmmInfo.id,
      lowerTickArrayStartIndex
    ).publicKey;
    const tick_array_upper = getPdaTickArrayAddress(
      DEVNET_PROGRAM_ID.CLMM,
      clmmInfo.id,
      upperTickArrayStartIndex
    ).publicKey;
    const personal_position = getPdaPersonalPositionAddress(
      DEVNET_PROGRAM_ID.CLMM,
      position_nft_mint.publicKey
    ).publicKey;
    const exTickArrayBitmap = PoolUtils.isOverflowDefaultTickarrayBitmap(
      clmmInfo.tickSpacing,
      [lowerTickArrayStartIndex, upperTickArrayStartIndex]
    )
      ? getPdaExBitmapAccount(clmmInfo.programId, clmmInfo.id).publicKey
      : undefined;
    const remainingAccounts = [
      { pubkey: position_nft_mint.publicKey, isSigner: true, isWritable: true },
      { pubkey: poolAndPositionNftAta, isSigner: false, isWritable: true },
      {
        pubkey: toWeb3JsPublicKey(position_nft_mint_metadata),
        isSigner: false,
        isWritable: true,
      },
      { pubkey: clmmInfo.id, isSigner: false, isWritable: true },
      { pubkey: protocol_position, isSigner: false, isWritable: true },
      { pubkey: tick_array_lower, isSigner: false, isWritable: true },
      { pubkey: tick_array_upper, isSigner: false, isWritable: true },
      { pubkey: personal_position, isSigner: false, isWritable: true },
      { pubkey: vaultAndToken0Ata, isSigner: false, isWritable: true },
      { pubkey: vaultAndToken1ATA, isSigner: false, isWritable: true },
      ...(exTickArrayBitmap
        ? [{ pubkey: exTickArrayBitmap, isSigner: false, isWritable: true }]
        : []),
    ];
    try {
      const tx = new Transaction();
      const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({
        units: 1000000,
      });
      tx.add(modifyComputeUnits);
      tx.add(
        await program.methods
          .launchTokenClmm(
            new BN(0),
            token0_amount,
            token1_amount,
            lowerPriceAndTick.tick,
            upperPriceAndTick.tick,
            lowerTickArrayStartIndex,
            upperTickArrayStartIndex
          )
          .accounts({
            payer: toWeb3JsPublicKey(signer.publicKey),
            raydiumProgram: DEVNET_PROGRAM_ID.CLMM,
            tokenProgram: TOKEN_PROGRAM_ID,
            associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
            metadataProgram: METADATA_PROGRAM_ID,
            tokenProgram2022: SPL_TOKEN_2022_PROGRAM_ID,
            rent: RENT_PROGRAM_ID,
            pool: poolId,
            tokenAccount0: payerAndToken0Ata,
            tokenAccount1: payerAndToken1Ata,
            poolTokenAccount0: poolAndToken0ATA,
            poolTokenAccount1: poolAndToken1ATA,
            vault0Mint: clmmInfo.mintA.mint,
            vault1Mint: clmmInfo.mintB.mint,
          })
          .remainingAccounts(remainingAccounts)
          .instruction()
      );
      await sendAndConfirmTransaction(program.provider.connection, tx, [
        toWeb3JsKeypair(signer),
        position_nft_mint,
      ]);
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
      rewardMint,
      toWeb3JsPublicKey(signer.publicKey),
      true
    );
    try {
      const txId = await program.methods
        .claimRewards()
        .accounts({
          purchaseReceipt: purchaseReceipt,
          pool: poolId,
          originalMint: nftA.mintAddress,
          payerOriginalMintAta: payerOriginalMintAta,
          rewardMint: rewardMint,
          payerRewardMintTokenAccount: payerRewardMintTokenAccount,
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
