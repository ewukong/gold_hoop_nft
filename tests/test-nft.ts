import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TestNft } from "../target/types/test_nft";
import { walletAdapterIdentity } from "@metaplex-foundation/umi-signer-wallet-adapters";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import {
  MPL_TOKEN_METADATA_PROGRAM_ID,
  findMasterEditionPda,
  findMetadataPda,
  mplTokenMetadata,
} from "@metaplex-foundation/mpl-token-metadata";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { publicKey } from "@metaplex-foundation/umi";
import fs from 'node:fs'
import { PublicKey } from '@solana/web3.js';
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";



describe("doge", async () => {
  // Configured the client to use the devnet cluster.
  const provider = anchor.AnchorProvider.env();


  anchor.setProvider(provider);
  const program = anchor.workspace
    .TestNft as Program<TestNft>;

  const signer = provider.wallet;


  const umi = createUmi(process.env.RPC_URL)
    .use(walletAdapterIdentity(signer))
    .use(mplTokenMetadata());

  const mint = anchor.web3.Keypair.generate();

  // Derive the associated token address account for the mint
  const associatedTokenAccount = await getAssociatedTokenAddress(
    mint.publicKey,
    signer.publicKey
  );

  const secretKeyString = fs.readFileSync(`/Users/biteagle/.config/solana/id.json`, { encoding: 'utf8' });
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  const root = anchor.web3.Keypair.fromSecretKey(secretKey);
  const secretKey11 = bs58.encode(root.secretKey)


  // derive the metadata account
  let metadataAccount = findMetadataPda(umi, {
    mint: publicKey(mint.publicKey),
  })[0];


  //derive the master edition pda
  let masterEditionAccount = findMasterEditionPda(umi, {
    mint: publicKey(mint.publicKey),
  })[0];

  const metadata = {
    name: "GoGoGo",
    symbol: "GGGG",
    uri: "https://bafkreiakb3s7nwtt7vdrqenv5rp5tkgdd2xoylcrsfqu7nghr5kizutkie.ipfs.nftstorage.link/",
  };

  const [merkleDataAccount] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      //We need to reference both objects as a Byte Buffer, which is what
      //Solana's find_program_address requires to find the PDA.
      Buffer.from("MerkleTokenDistributor"),
      signer.publicKey.toBuffer(),
    ],
    program.programId,
  );

  const [nftData] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      //We need to reference both objects as a Byte Buffer, which is what
      //Solana's find_program_address requires to find the PDA.
      Buffer.from("nftData"),
      signer.publicKey.toBuffer(),
    ],
    program.programId,
  );


  // it("initdata!!!", async () => {
  //   console.log(associatedTokenAccount.toString(), '\n=---=', mint.publicKey.toString());

  //   const tx = await program.methods
  //     .initialize()
  //     .accounts({
  //       merkleDataAccount,
  //       nftData,
  //       signer: root.publicKey,
  //     })
  //     .signers([root])
  //     .rpc()

  //   console.log(
  //     `mint nft tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`
  //   );
  //   console.log(
  //     `minted nft: https://explorer.solana.com/address/${mint.publicKey}?cluster=devnet`
  //   );
  // });

  it("mints nft!!!!", async () => {
    const tx = await program.methods
      .mint(metadata.name, metadata.symbol, metadata.uri, new PublicKey('5xCHpu1AsFhKe3fkiWVwvgt5dSW87W5rsryVXrHwESv3'))
      .accounts({
        signer: provider.publicKey,
        mint: mint.publicKey,
        metadataAccount,
        masterEditionAccount,
        collectionId: new PublicKey('5xCHpu1AsFhKe3fkiWVwvgt5dSW87W5rsryVXrHwESv3'),
        associatedTokenAccount,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenMetadataProgram: MPL_TOKEN_METADATA_PROGRAM_ID,
        merkleDataAccount,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        nftData: nftData,
      })
      .signers([mint])
      .rpc()

    console.log(
      `mint nft tx: https://explorer.solana.com/tx/${tx}?cluster=devnet`
    );
    console.log(
      `minted nft: https://explorer.solana.com/address/${mint.publicKey}?cluster=devnet`
    );
  });
});