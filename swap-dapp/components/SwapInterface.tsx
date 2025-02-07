"use client"

import React, { useState } from 'react';
import { Card, CardHeader, CardContent, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';

import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice } from "@cosmjs/stargate";

import { Window as KeplrWindow } from "@keplr-wallet/types";

declare global {
  interface Window extends KeplrWindow {}
}

const SwapInterface = () => {
  const [account, setAccount] = useState('');
  const [amount, setAmount] = useState('');
  const [recipient, setRecipient] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  // Neutron testnet configuration
  const CONTRACT_ADDRESS = "neutron1d55f2u4rm4l7fy9l72crwhr9cwcsv4n0pggggcvf5jzkwtrvdlhsthasyg";
  const CHAIN_ID = "pion-1";
  const RPC_ENDPOINT = "https://rpc-palvus.pion-1.ntrn.tech";

  // Chain configuration for Keplr
  const chainConfig = {
    chainId: CHAIN_ID,
    chainName: 'Neutron Testnet',
    rpc: RPC_ENDPOINT,
    rest: 'https://rest-palvus.pion-1.ntrn.tech',
    bip44: {
      coinType: 118,
    },
    bech32Config: {
      bech32PrefixAccAddr: 'neutron',
      bech32PrefixAccPub: 'neutronpub',
      bech32PrefixValAddr: 'neutronvaloper',
      bech32PrefixValPub: 'neutronvaloperpub',
      bech32PrefixConsAddr: 'neutronvalcons',
      bech32PrefixConsPub: 'neutronvalconspub',
    },
    currencies: [
      {
        coinDenom: 'NTRN',
        coinMinimalDenom: 'untrn',
        coinDecimals: 6,
      },
    ],
    feeCurrencies: [
      {
        coinDenom: 'NTRN',
        coinMinimalDenom: 'untrn',
        coinDecimals: 6,
      },
    ],
    stakeCurrency: {
      coinDenom: 'NTRN',
      coinMinimalDenom: 'untrn',
      coinDecimals: 6,
    },
    gasPrices: '0.025untrn',
    gasAdjustment: 1.3,
  };

  // Connect to Keplr wallet
  const connectWallet = async () => {
    try {
      if (!window.keplr) {
        throw new Error("Please install Keplr extension");
      }

      // Suggest chain to Keplr
      await window.keplr.experimentalSuggestChain(chainConfig);
      
      // Enable access to Keplr
      await window.keplr.enable(CHAIN_ID);
      
      const key = await window.keplr.getKey(CHAIN_ID);
      setAccount(key.bech32Address);
    //   setKeplr(window.keplr?);
      
    } catch (error) {
        console.error(error)
    //   setError(error);
    }
  };

  // Execute swap transaction
  const executeSwap = async () => {
    try {
        setLoading(true);
        setError('');

        if (!window.keplr || !account) {
            throw new Error("Please connect your wallet first");
        }

        const formatted_amount = parseInt(amount) * 1000000;
        const string_formatted_amount = formatted_amount.toString();

        // Prepare the swap message
        const msg = {
            swap: {
                recipient: recipient || account,
                amount_in: string_formatted_amount
            }
        };

        // Create a CosmWasm client
        try {
            // Get offline signer from Keplr
            const offlineSigner = window.keplr.getOfflineSigner(CHAIN_ID);

            // Create a CosmWasm client
            const client = await SigningCosmWasmClient.connectWithSigner(
                RPC_ENDPOINT,
                offlineSigner,
                { gasPrice: GasPrice.fromString("0.025untrn") }
            );

            // Retrieve account information from the signer
            const accounts = await offlineSigner.getAccounts();
            const address = accounts[0].address;

            console.log("address: ", address);

            // Execute the transaction
            const result = await client.execute(
                account,
                CONTRACT_ADDRESS,
                msg,
                "auto",
                undefined,
                [
                    {
                        denom: "untrn",
                        amount: string_formatted_amount
                    }
                ]
            );

            console.log("Swap successful:", result);
            setLoading(false);
        } catch (broadcastError) {
            console.error(broadcastError);
            // throw new Error(`Failed to broadcast: ${broadcastError.message}`);
        }      
    } catch (error) {
    // setError(error.message);
      setLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-lg mx-auto mt-8 sm:w-full">
      <CardHeader>
        <CardTitle>Neutron Testnet Donation</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {!account ? (
            <Button 
              onClick={connectWallet}
              className="w-full"
            >
              Connect Keplr Wallet
            </Button>
          ) : (
            <Alert>
              <AlertDescription>
                Connected: {account.slice(0, 8)}...{account.slice(-8)}
              </AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <Label htmlFor="amount">Amount (at least 1)</Label>
            <Input
              id="amount"
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="Enter amount"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="recipient">Recipient (optional)</Label>
            <Input
              id="recipient"
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              placeholder="neutron..."
            />
          </div>

          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <Button
            onClick={executeSwap}
            disabled={!account || loading || !amount}
            className="w-full"
          >
            {loading ? "Processing..." : "Swap"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};

export default SwapInterface;