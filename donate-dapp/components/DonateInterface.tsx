"use client";

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

const DonateInterface = () => {
  const [account, setAccount] = useState('');
  const [amount, setAmount] = useState('');
  const [recipient, setRecipient] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [success, setSuccess] = useState('');

  // Neutron testnet configuration
  const CONTRACT_ADDRESS = "neutron1ukzxaw7s83ej38sk2kdf2f5sam60uexeganwlxdksj7x4us4js6sa4ekw6";
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
    } catch (error) {
      console.error(error);
      // setError(error.message);
    }
  };

  // Execute donation transaction
  const executeDonation = async () => {
    try {
        setLoading(true);
        setError('');
        setSuccess('');

        if (!window.keplr || !account) {
            throw new Error("Please connect your wallet first");
        }

        const formatted_amount = parseInt(amount) * 1000000;
        if (formatted_amount <= 0) {
          throw new Error("Amount must be greater than 0");
        }
        const string_formatted_amount = formatted_amount.toString();

        // Prepare the donation message
        const msg = {
            donate: {
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

            console.log("Donation successful:", result);
            setSuccess("Donation was successful!");
            setLoading(false);
        } catch (broadcastError) {
            console.error(broadcastError);
            setError("Failed to broadcast transaction. Please try again.");
            setLoading(false);
        }      
    } catch (error) {
      console.error(error);
      // setError(error.message);
      setLoading(false);
    }
  };

  return (
    <Card className="w-full max-w-lg mx-auto mt-8 sm:w-full bg-gray-100">
      <CardHeader>
        <CardTitle className="text-pink-600">Donate to a Cause</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {!account ? (
            <Button 
              onClick={connectWallet}
              className="w-full bg-pink-500 hover:bg-pink-700 text-white"
            >
              Connect Keplr Wallet
            </Button>
          ) : (
            <Alert className="bg-green-100 text-green-800">
              <AlertDescription>
                Connected: {account.slice(0, 8)}...{account.slice(-8)}
              </AlertDescription>
            </Alert>
          )}

          <div className="space-y-2">
            <Label htmlFor="amount" className="text-pink-600">Donation Amount (at least 1)</Label>
            <Input
              id="amount"
              type="number"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              placeholder="Enter amount"
              className="border-pink-500"
            />
          </div>

          <div className="space-y-2">
            <Label htmlFor="recipient" className="text-pink-600">Recipient (optional)</Label>
            <Input
              id="recipient"
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              placeholder="Recipient address"
              className="border-pink-500"
            />
          </div>

          {error && (
            <Alert variant="destructive" className="bg-red-100 text-red-800">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {success && (
            <Alert className="bg-green-100 text-green-800">
              <AlertDescription>{success}</AlertDescription>
            </Alert>
          )}

          <Button
            onClick={executeDonation}
            disabled={!account || loading || !amount}
            className="w-full bg-pink-500 hover:bg-pink-700 text-white"
          >
            {loading ? "Processing..." : "Donate"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};

export default DonateInterface;
