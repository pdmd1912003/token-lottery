import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as sb from '@switchboard-xyz/on-demand';
import { TokenLottery } from "../target/types/token_lottery";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import SwitchBoardIDL from '../switchboard.json';
import {
  Connection,
  Keypair,
  PublicKey,
  SystemProgram,
  Commitment,
} from "@solana/web3.js";


describe("token-lottery", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const wallet = provider.wallet as anchor.Wallet;

  const program = anchor.workspace.tokenLottery as Program<TokenLottery>;
  let switchboardProgram;

  const rngKp = anchor.web3.Keypair.generate();

  before("Loading switchboard program", async () => {
    const switchboardIDL = await anchor.Program.fetchIdl(
      sb.ON_DEMAND_MAINNET_PID, 
      {connection: new anchor.web3.Connection("https://api.mainnet-beta.solana.com")}
    );
    switchboardProgram = new anchor.Program(switchboardIDL, provider);
  });

  async function buyTicket() {
    const buyTicketIx = await program.methods.buyTicket()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .instruction();

    const blockhashContext = await provider.connection.getLatestBlockhash();

    const computeIx = anchor.web3.ComputeBudgetProgram.setComputeUnitLimit({
      units: 300000
    });

    const priorityIx = anchor.web3.ComputeBudgetProgram.setComputeUnitPrice({
      microLamports: 1
    });

    const tx = new anchor.web3.Transaction({
      blockhash: blockhashContext.blockhash,
      lastValidBlockHeight: blockhashContext.lastValidBlockHeight,
      feePayer: wallet.payer.publicKey,
    }).add(buyTicketIx)
      .add(computeIx)
      .add(priorityIx);

    const sig = await anchor.web3.sendAndConfirmTransaction(provider.connection, tx, [wallet.payer], { skipPreflight: true });
    console.log("buy ticket ", sig);
  }


  it("should init config!", async () => {
    const initConfigTx = await program.methods
      .initializeConfig(
        new anchor.BN(0),
        new anchor.BN(17852469070),
        new anchor.BN(1000000)
      ).instruction();

    const blockhashWithContext = await provider.connection.getLatestBlockhash();

    const tx = new anchor.web3.Transaction(
      {
        feePayer: provider.wallet.publicKey,
        blockhash: blockhashWithContext.blockhash,
        lastValidBlockHeight: blockhashWithContext.lastValidBlockHeight,
      }
    ).add(initConfigTx);

    const signature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection, tx, [wallet.payer]
    );

    console.log('Your transaction signature:', signature);

    const initLotteryIx = await program.methods.initializeLottery().accounts({
      tokenProgram: TOKEN_PROGRAM_ID,
    }).instruction();

    const initLotteryTx = new anchor.web3.Transaction({
      feePayer: provider.wallet.publicKey,
      blockhash: blockhashWithContext.blockhash,
      lastValidBlockHeight: blockhashWithContext.lastValidBlockHeight,
    }).add(initLotteryIx);

    const initLotterySignature = await anchor.web3.sendAndConfirmTransaction(
      provider.connection, initLotteryTx, [wallet.payer], { skipPreflight: true }
    );
    console.log('Init lottery signature:', initLotterySignature);


    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();
    await buyTicket();

    console.log("Waiting for 4 seconds before creating randomness...");
    await new Promise(resolve => setTimeout(resolve, 4000));


   


    // const queue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w");

    // const queueAccount = new sb.Queue(switchboardProgram, queue);

    const queueAccount = await setupQueue(switchboardProgram);
    console.log("Queue account", queueAccount.toBase58());
    // try {
    //   await queueAccount.loadData();
    // } catch (err) {
    //   console.log("Queue account not found");
    //   process.exit(1);
    // }

    const [randomness, createRandomnessIx] = await sb.Randomness.create(switchboardProgram, rngKp, queueAccount);
    console.log("Created randomness account..");
    console.log("Randomness account", randomness.pubkey.toBase58());
    const createRandomnessTx = await sb.asV0Tx({
      connection: provider.connection,
      ixs: [createRandomnessIx],
      payer: wallet.publicKey,
      signers: [wallet.payer, rngKp],
    })
    const createRandomnessSignature = await provider.connection.sendTransaction(createRandomnessTx);
    console.log('Create randomness signature:', createRandomnessSignature);

  });


});

 export async function setupQueue(program: anchor.Program): Promise<PublicKey> {
      const queueAccount = await sb.getDefaultQueue(
        program.provider.connection.rpcEndpoint
      );
      console.log("Queue account", queueAccount.pubkey.toString());
      try {
        await queueAccount.loadData();
      } catch (err) {
        console.error("Queue not found, ensure you are using devnet in your env");
        process.exit(1);
      }
      return queueAccount.pubkey;
    }