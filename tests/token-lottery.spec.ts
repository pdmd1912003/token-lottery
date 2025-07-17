import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import * as sb from '@switchboard-xyz/on-demand';
import { TokenLottery } from "../target/types/token_lottery";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import SwitchBoardIDL from '../swichboard.json';

describe("token-lottery", () => {
  const provider = anchor.AnchorProvider.env();
  const connection = provider.connection;
  anchor.setProvider(provider);
  const wallet = provider.wallet as anchor.Wallet;
  
  const program = anchor.workspace.tokenLottery as Program<TokenLottery>;

  let swichboardProgram = new anchor.Program(SwitchBoardIDL as anchor.Idl, provider);
  const rngKp = anchor.web3.Keypair.generate();

  async function buyTicket() {
    const buyTicketIx = await program.methods.buyTicket()
      .accounts({
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .instruction();

    const blockhashContext = await connection.getLatestBlockhash();

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

    const sig = await anchor.web3.sendAndConfirmTransaction(connection, tx, [wallet.payer], {skipPreflight: true});
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
      connection, initLotteryTx, [wallet.payer], {skipPreflight: true}
    );
    console.log('Init lottery signature:', initLotterySignature);
    

    await buyTicket();

    // const queue = new anchor.web3.PublicKey("A43DyUGA7s8eXPxqEjJY6EBu1KKbNgfxF8h17VAHn13w");
  });


});
