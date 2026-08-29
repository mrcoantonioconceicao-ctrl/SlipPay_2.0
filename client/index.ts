import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { SolanaSandboxCounter } from "../target/types/solana_sandbox_counter";

describe("solana_sandbox_counter", () => {
  // Configure the client to use the local devnet/cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaSandboxCounter as Program<SolanaSandboxCounter>;
  const authority = provider.wallet;

  // 1. Derive PDA for UserCounter
  const [counterPda, bump] = PublicKey.findProgramAddressSync(
    [Buffer.from("counter"), authority.publicKey.toBuffer()],
    program.programId
  );

  it("Initializes the PDA Counter Account", async () => {
    console.log("Authority Pubkey:", authority.publicKey.toBase58());
    console.log("Derived PDA Counter:", counterPda.toBase58());

    const tx = await program.methods
      .initialize()
      .accounts({
        counter: counterPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Transaction Signature:", tx);

    // Fetch account state from chain
    const counterAccount = await program.account.userCounter.fetch(counterPda);
    console.log("On-Chain Counter State:", {
      authority: counterAccount.authority.toBase58(),
      count: counterAccount.count.toNumber(),
      bump: counterAccount.bump,
    });
  });

  it("Increments the Counter", async () => {
    const tx = await program.methods
      .increment()
      .accounts({
        counter: counterPda,
        authority: authority.publicKey,
      })
      .rpc();

    console.log("Increment Tx Signature:", tx);

    const counterAccount = await program.account.userCounter.fetch(counterPda);
    console.log("Updated Count:", counterAccount.count.toNumber());
  });
});
