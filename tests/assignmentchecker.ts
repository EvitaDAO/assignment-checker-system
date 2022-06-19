import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Assignmentchecker } from "../target/types/assignmentchecker";

describe("assignmentchecker", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.Assignmentchecker as Program<Assignmentchecker>;

  it("creates assignment checker!", async () => {
    const authorityKeypair = anchor.web3.Keypair.generate();
    const checker_authority = (program.provider as anchor.AnchorProvider).wallet;
    const tx = await program.methods.create(10, 100, [], []).rpc();
    console.log("Your transaction signature", tx);
  });
});
