import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";
import dotenv from "dotenv";

dotenv.config();

const rpcEndpoint = "https://rpc-palvus.pion-1.ntrn.tech";
const mnemonic = process.env.MNEMONIC;
const wasmFilePath = "../artifacts/hackatom_faucet.wasm";

async function main() {
  try {
    // Create wallet from mnemonic
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
      prefix: "neutron",
    });

    const [firstAccount] = await wallet.getAccounts();

    // Connect client with signer
    const client = await SigningCosmWasmClient.connectWithSigner(
      rpcEndpoint,
      wallet,
      {
        gasPrice: GasPrice.fromString("0.025untrn"),
      }
    );

    // Upload contract code
    const wasmCode = readFileSync(wasmFilePath);
    const uploadReceipt = await client.upload(
      firstAccount.address,
      wasmCode,
      "auto"
    );
    console.log("Upload successful, code ID:", uploadReceipt.codeId);

    // Configure instantiation message matching your contract's InstantiateMsg
    const initMsg = {
      admin: firstAccount.address, // Using deployer's address as admin
      tokens: [
        {
          denom: { native: "untrn" }, 
          amount: "100000"         // 0.1 token (considering 6 decimals)
        },
        {
          denom: { cw20: "neutron1sr60e2velepytzsdyuutcmccl9n2p2lu3pjcggllxyc9rzyu562sqegazj" }, // tATOM
          amount: "100000000" // 100 token (considering 6 decimals)
        },
        {
          denom: { cw20: "neutron1he6zd5kk03cs5ywxk5tth9qfewxwnh7k9hjwekr7gs9gl9argadsqdc9rp" }, // tNGN
          amount: "1000000"
        }
      ],
      rate_limit_seconds: 86400 // 24 hours as example
    };

    // Instantiate the contract
    const instantiateReceipt = await client.instantiate(
      firstAccount.address,
      uploadReceipt.codeId,
      initMsg,
      "Naija Hackatom Faucet",
      "auto"
    );

    console.log("Contract instantiated at:", instantiateReceipt.contractAddress);
    console.log("Transaction hash:", instantiateReceipt.transactionHash);

  } catch (error) {
    console.error("Deployment failed:", error);
  }
}

main();

// Upload successful, code ID: 10966
// Contract instantiated at: neutron1w0ls4rscaug5gz30envteezu2yfug2ychs3ts204rzmrfr66g7dss2qlt7
// Transaction hash: ADCA40130A0A4725488860A4327540050E4A33C96AC6F715D4750C0C9BCCC673