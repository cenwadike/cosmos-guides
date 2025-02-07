import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { readFileSync } from "fs";

import dotenv from "dotenv"

dotenv.config()

const rpcEndpoint = "https://rpc-palvus.pion-1.ntrn.tech";
const mnemonic = process.env.MNEMONIC;
const wasmFilePath = "./artifacts/first_token_cw20contract.wasm";

async function main() {
  const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
    prefix: "neutron",
  });

  const [firstAccount] = await wallet.getAccounts();

  const client = await SigningCosmWasmClient.connectWithSigner(
    rpcEndpoint,
    wallet,
    {
      gasPrice: GasPrice.fromString("0.025untrn"),
    }
  );

  const wasmCode = readFileSync(wasmFilePath);
  const uploadReceipt = await client.upload(firstAccount.address, wasmCode, "auto");
  console.log("Upload successful, code ID:", uploadReceipt.codeId);

  const initMsg = {
    name: "Token Two",
    symbol: "TKNTwo",
    decimals: 6,
    initial_balances: [
      {
        "address": "neutron1tn5uf2q6n5ks8a40vkf2j2tkz0c9asd0udq6t4",
        "amount": "10000000"
      },
      {
        "address": "neutron1zvh7g2gjk3e4tac7f3gq9u4fwjrkhpcanqp2exa5q5utmn594nqq3z05py",
        "amount": "10000000"
      }
    ]
  };

  const instantiateReceipt = await client.instantiate(firstAccount.address, uploadReceipt.codeId, initMsg, "CW Token", "auto");
  console.log("Contract instantiated at:", instantiateReceipt.contractAddress);
}

main().catch(console.error);


