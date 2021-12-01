import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { StakingPool } from '../target/types/staking_pool';

describe('staking-pool', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.StakingPool as Program<StakingPool>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
