import { PublicKey } from "@solana/web3.js";

export const TOKEN_METADATA_PROGRAM_ID = new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");

export function getPdaMetadata(mint: PublicKey): PublicKey {
  const [metadata] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
  return metadata;
}

export function getPdaMasterEdition(mint: PublicKey): PublicKey {
  const [masterEdition] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      TOKEN_METADATA_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from("edition"),
    ],
    TOKEN_METADATA_PROGRAM_ID
  );
  return masterEdition;
}

export function getCollectionTokenAccount(collectionMint: PublicKey, authority: PublicKey): PublicKey {
  const [collectionTokenAccount] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("collection_token_account"),
      authority.toBuffer()
    ],
    collectionMint
  );
  return collectionTokenAccount;
}
