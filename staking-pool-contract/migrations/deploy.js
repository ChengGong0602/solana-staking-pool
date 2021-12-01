// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

const anchor = require("@project-serum/anchor");
const { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, Token } = require('@solana/spl-token')
const { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Keypair } = anchor.web3;

const idl = require("../target/idl/staking_pool.json");
const programID = idl.metadata.address;
const program_title = "staking_05";
const pool_seed = "pool_seeded";

// SEEDED MINT
const seededMint = new PublicKey("Gag5A5jjD38CX93QF828oNJQQgoYeuKBNpwcKqwx3oG4");

module.exports = async function (provider) {
  // Configure client to use the provider.
  anchor.setProvider(provider);

  // Add your deploy script here.
  const program = new anchor.Program(idl, programID);

  try {
    /* interact with the program via rpc */
    let bumps = {
      storeAccount: 0,
      poolSeeded: 0,
    };

    // Find PDA from `seed` for state account
    const [storeAccount, storeAccountBump] = await PublicKey.findProgramAddress(
      [Buffer.from(program_title)],
      program.programId
    );
    bumps.storeAccount = storeAccountBump;
    // Find PDA from `seed` for staking pool
    const [poolSeeded, poolSeededBump] = await PublicKey.findProgramAddress(
      [Buffer.from(program_title), Buffer.from(pool_seed)],
      program.programId
    );
    bumps.poolSeeded = poolSeededBump;

    console.log("StoreAccount", storeAccount.toBase58());
    console.log("Pool-SEEDED", poolSeeded.toBase58());
    console.log("Bumps", bumps);

    // Signer
    const storeAuthority = provider.wallet.publicKey;

    // initialize
    await program.rpc.initialize(program_title, bumps, {
      accounts: {
        storeAuthority,
        storeAccount,
        seededMint,
        poolSeeded,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      },
    });

  } catch (err) {
    console.log("Transaction error: ", err);
  }
}

// StoreAccount 5M6h6KKVrJX786RrMQQvngSE2NDmGDo8JRmTZDiRccUG
// Pool-SEEDED CKxPVPw8PzQKnXW5fVgUzKCpYAYH1467N1jebDT9ZqZj
// Bumps { storeAccount: 253, poolSeeded: 255 }