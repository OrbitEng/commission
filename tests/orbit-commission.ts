import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { OrbitCommission } from "../target/types/orbit_commission";

describe("orbit-commission", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.OrbitCommission as Program<OrbitCommission>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
