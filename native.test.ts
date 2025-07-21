describe("Dice Game", () => {
  it("Player rolls a dice", async () => {
    const player = pg.wallet.keypair;
    const vaultSeed = Buffer.from("vault");

    const [vaultPda] = await web3.PublicKey.findProgramAddress(
      [vaultSeed],
      pg.PROGRAM_ID
    );

    // 🎯 Bet params
    const bet = 10_000_000;      // 0.01 SOL
    const target = 50.5;         // Roll under or over 74.5
    const isOver = 1;            // 0 = under, 1 = over

    // 🎲 Build 11-byte instruction data
    const data = Buffer.alloc(11);
    data.writeBigUInt64LE(BigInt(bet), 0);               // bet
    data.writeUInt16LE(Math.floor(target * 10), 8);      // target * 10
    data.writeUInt8(isOver, 10);                         // over/under

    const rollIx = new web3.TransactionInstruction({
      programId: pg.PROGRAM_ID,
      keys: [
        { pubkey: player.publicKey, isSigner: true, isWritable: true },
        { pubkey: vaultPda, isSigner: false, isWritable: true },
        { pubkey: web3.SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      data,
    });

    const tx = new web3.Transaction().add(rollIx);
    const sig = await web3.sendAndConfirmTransaction(pg.connection, tx, [player]);

    console.log("🎲 Roll sent! Signature:", sig);

    const balance = await pg.connection.getBalance(player.publicKey);
    console.log("💰 Player balance:", balance / web3.LAMPORTS_PER_SOL, "SOL");
  });
});
