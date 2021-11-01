import * as anchor from "@project-serum/anchor";
import { Program, Provider, web3 } from "@project-serum/anchor";
import { expect } from "chai";

describe("SplTokenGen", () => {
  const provider = anchor.Provider.env();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  it("Is initialized!", async () => {
    const program = anchor.workspace.SplTokenGen as Program;

    const payer = web3.Keypair.generate();

    const airtx = await provider.connection.requestAirdrop(
      payer.publicKey,
      web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airtx);

    const [storageReference, storageReferenceBumpSeed] =
      await web3.PublicKey.findProgramAddress(
        [Buffer.from("key"), payer.publicKey.toBuffer(), Buffer.from("foo")],
        program.programId
      );

    const [initialStorage, initialStorageBumpSeed] =
      await web3.PublicKey.findProgramAddress(
        [Buffer.from("init"), storageReference.toBuffer()],
        program.programId
      );

    await program.rpc.init(Buffer.from("foo"), storageReferenceBumpSeed, {
      accounts: {
        payer: payer.publicKey,
        storageReference: storageReference,
        initialStorage: initialStorage,
        systemProgram: web3.SystemProgram.programId,
      },
      signers: [payer],
    });

    const data = await getStorage(provider, program, storageReference);
    expect(data).to.be.eq(null);

    const [nextStorage, nextStorageBumpSeed] =
      await web3.PublicKey.findProgramAddress(
        [Buffer.from("next"), initialStorage.toBuffer()],
        program.programId
      );

    await program.rpc.set(Buffer.from("bar"), {
      accounts: {
        payer: payer.publicKey,
        storageReference: storageReference,
        storage: initialStorage,
        nextStorage: nextStorage,
        systemProgram: web3.SystemProgram.programId,
      },
      signers: [payer],
    });

    const data2 = await getStorage(provider, program, storageReference);
    expect(data2).to.deep.eq(Buffer.from("bar"));

    const [nextStorage2, nextStorageBumpSeed2] =
      await web3.PublicKey.findProgramAddress(
        [Buffer.from("next"), nextStorage.toBuffer()],
        program.programId
      );

    await program.rpc.clear({
      accounts: {
        payer: payer.publicKey,
        storageReference: storageReference,
        storage: nextStorage,
        nextStorage: nextStorage2,
        systemProgram: web3.SystemProgram.programId,
      },
      signers: [payer],
    });

    const data3 = await getStorage(provider, program, storageReference);
    expect(data3).to.be.eq(null);
  });
});

async function getStorage(
  provider: Provider,
  program: Program,
  storageReference: web3.PublicKey
) {
  const storageReferenceValue = await program.account.storageReference.fetch(
    storageReference
  );
  const storage = storageReferenceValue.storage;

  const storageAccountInfo = await provider.connection.getAccountInfo(storage);

  if (storageAccountInfo !== null) {
    const data = storageAccountInfo.data;

    expect(data.length).to.be.gt(0);

    const header = data[0];
    const payload = data.slice(1);
    if (header === 0) {
      return null;
    } else {
      return payload;
    }
  }

  return null;
}
