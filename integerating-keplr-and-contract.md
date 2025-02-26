# Integrating Keplr Wallet and CosmWasm Smart Contracts: A Comprehensive Guide

## Table of Contents
1. [Prerequisites](#prerequisites)
2. [Project Setup](#project-setup)
3. [Keplr Wallet Integration](#keplr-wallet-integration)
4. [CosmWasm Contract Integration](#cosmwasm-contract-integration)
5. [Building the UI Components](#building-the-ui-components)
6. [Testing and Deployment](#testing-and-deployment)

## Prerequisites

Before starting, ensure you have the following installed:
- Node.js (v14 or higher)
- npm or yarn
- A CosmWasm-compatible blockchain network (e.g., Osmosis, Juno)
- Keplr wallet browser extension

## Project Setup

First, create a new React project using Create React App:

```bash
npx create-react-app keplr-cosmwasm-demo
cd keplr-cosmwasm-demo
npm install @keplr-wallet/types @cosmjs/cosmwasm-stargate @cosmjs/proto-signing
```

Create the following project structure:
```
src/
  ├── components/
  │   ├── WalletConnect.tsx
  │   ├── ContractInteraction.tsx
  │   └── TokenBalance.tsx
  ├── hooks/
  │   ├── useKeplr.ts
  │   └── useContract.ts
  ├── config/
  │   └── chain.ts
  └── App.tsx
```

## Keplr Wallet Integration

### 1. Chain Configuration

Create `src/config/chain.ts`:

```typescript
export const chainConfig = {
  chainId: "juno-1",
  chainName: "Juno",
  rpc: "https://rpc-juno.itastakers.com",
  rest: "https://lcd-juno.itastakers.com",
  bip44: {
    coinType: 118,
  },
  bech32Config: {
    bech32PrefixAccAddr: "juno",
    bech32PrefixAccPub: "junopub",
    bech32PrefixValAddr: "junovaloper",
    bech32PrefixValPub: "junovaloperpub",
    bech32PrefixConsAddr: "junovalcons",
    bech32PrefixConsPub: "junovalconspub",
  },
  currencies: [
    {
      coinDenom: "JUNO",
      coinMinimalDenom: "ujuno",
      coinDecimals: 6,
    },
  ],
  feeCurrencies: [
    {
      coinDenom: "JUNO",
      coinMinimalDenom: "ujuno",
      coinDecimals: 6,
    },
  ],
  stakeCurrency: {
    coinDenom: "JUNO",
    coinMinimalDenom: "ujuno",
    coinDecimals: 6,
  },
  gasPriceStep: {
    low: 0.01,
    average: 0.025,
    high: 0.04,
  },
};
```

### 2. Keplr Hook

Create `src/hooks/useKeplr.ts`:

```typescript
import { useState, useCallback } from 'react';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { chainConfig } from '../config/chain';

export const useKeplr = () => {
  const [address, setAddress] = useState<string>('');
  const [client, setClient] = useState<SigningCosmWasmClient | null>(null);

  const connect = useCallback(async () => {
    if (!window.keplr) {
      alert('Please install Keplr extension');
      return;
    }

    try {
      // Enable the chain in Keplr
      await window.keplr.enable(chainConfig.chainId);

      // Get the offlineSigner for signing transactions
      const offlineSigner = window.keplr.getOfflineSigner(chainConfig.chainId);

      // Get user address
      const accounts = await offlineSigner.getAccounts();
      setAddress(accounts[0].address);

      // Create signing client
      const client = await SigningCosmWasmClient.connectWithSigner(
        chainConfig.rpc,
        offlineSigner
      );
      setClient(client);
    } catch (error) {
      console.error('Error connecting to Keplr:', error);
      alert('Failed to connect to Keplr');
    }
  }, []);

  return { address, client, connect };
};
```

## CosmWasm Contract Integration

### 1. Contract Hook

Create `src/hooks/useContract.ts`:

```typescript
import { useCallback } from 'react';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';

export const useContract = (
  client: SigningCosmWasmClient | null,
  contractAddress: string
) => {
  const queryContract = useCallback(
    async (queryMsg: Record<string, unknown>) => {
      if (!client) return null;
      
      try {
        const result = await client.queryContractSmart(contractAddress, queryMsg);
        return result;
      } catch (error) {
        console.error('Error querying contract:', error);
        return null;
      }
    },
    [client, contractAddress]
  );

  const executeContract = useCallback(
    async (
      senderAddress: string,
      executeMsg: Record<string, unknown>,
      funds?: readonly Coin[]
    ) => {
      if (!client) return null;

      try {
        const result = await client.execute(
          senderAddress,
          contractAddress,
          executeMsg,
          'auto',
          undefined,
          funds
        );
        return result;
      } catch (error) {
        console.error('Error executing contract:', error);
        return null;
      }
    },
    [client, contractAddress]
  );

  return { queryContract, executeContract };
};
```

## Building the UI Components

### 1. Wallet Connect Component

Create `src/components/WalletConnect.tsx`:

```typescript
import React from 'react';
import { useKeplr } from '../hooks/useKeplr';

export const WalletConnect: React.FC = () => {
  const { address, connect } = useKeplr();

  return (
    <div>
      {!address ? (
        <button onClick={connect}>Connect Wallet</button>
      ) : (
        <div>
          <p>Connected Address: {address}</p>
        </div>
      )}
    </div>
  );
};
```

### 2. Contract Interaction Component

Create `src/components/ContractInteraction.tsx`:

```typescript
import React, { useState } from 'react';
import { useContract } from '../hooks/useContract';
import { useKeplr } from '../hooks/useKeplr';

interface ContractInteractionProps {
  contractAddress: string;
}

export const ContractInteraction: React.FC<ContractInteractionProps> = ({
  contractAddress,
}) => {
  const { address, client } = useKeplr();
  const { queryContract, executeContract } = useContract(client, contractAddress);
  const [queryResult, setQueryResult] = useState<any>(null);

  const handleQuery = async () => {
    const result = await queryContract({ 
      get_count: {} // Example query message
    });
    setQueryResult(result);
  };

  const handleExecute = async () => {
    if (!address) return;

    const result = await executeContract(
      address,
      { increment: {} }, // Example execute message
      []
    );
    
    if (result) {
      alert('Transaction successful!');
      handleQuery(); // Refresh the query result
    }
  };

  return (
    <div>
      <h2>Contract Interaction</h2>
      <button onClick={handleQuery}>Query Contract</button>
      {queryResult && (
        <pre>{JSON.stringify(queryResult, null, 2)}</pre>
      )}
      <button onClick={handleExecute} disabled={!address}>
        Execute Contract
      </button>
    </div>
  );
};
```

### 3. App Component

Update `src/App.tsx`:

```typescript
import React from 'react';
import { WalletConnect } from './components/WalletConnect';
import { ContractInteraction } from './components/ContractInteraction';

const CONTRACT_ADDRESS = 'your-contract-address-here';

function App() {
  return (
    <div className="App">
      <h1>Keplr + CosmWasm Demo</h1>
      <WalletConnect />
      <ContractInteraction contractAddress={CONTRACT_ADDRESS} />
    </div>
  );
}

export default App;
```

## Testing and Deployment

1. **Local Testing**

```bash
npm start
```

2. **Production Build**

```bash
npm run build
```

3. **Testing Checklist**
   - Wallet connection works
   - Contract queries return expected results
   - Contract executions complete successfully
   - Error handling works as expected
   - Transaction feedback is clear to users

## Error Handling Best Practices

1. Always check for Keplr availability:
```typescript
if (!window.keplr) {
  alert('Please install Keplr extension');
  return;
}
```

2. Handle transaction failures:
```typescript
try {
  const result = await executeContract(...);
  if (!result) throw new Error('Transaction failed');
  // Handle success
} catch (error) {
  console.error('Transaction error:', error);
  alert('Transaction failed: ' + error.message);
}
```

3. Implement loading states:
```typescript
const [isLoading, setIsLoading] = useState(false);

const handleTransaction = async () => {
  setIsLoading(true);
  try {
    await executeContract(...);
  } finally {
    setIsLoading(false);
  }
};
```

## Security Considerations

1. **Input Validation**: Always validate user input before sending it to the contract
2. **Gas Estimation**: Implement proper gas estimation for transactions
3. **Error Messages**: Don't expose sensitive information in error messages
4. **Transaction Confirmation**: Always wait for transaction confirmation before updating UI
5. **Network Security**: Use secure RPC endpoints and SSL/TLS connections

## Additional Features to Consider

1. **Transaction History**
2. **Balance Display**
3. **Network Selector**
4. **Gas Fee Customization**
5. **Contract Event Monitoring**

Remember to replace placeholder values like `your-contract-address-here` with actual values from your deployment.

For production applications, consider implementing additional features like:
- Loading indicators
- Better error handling
- Transaction history
- Network switching
- Gas fee customization

## Resources

- [Keplr Wallet Documentation](https://docs.keplr.app/)
- [CosmJS Documentation](https://cosmos.github.io/cosmjs/)
- [CosmWasm Documentation](https://docs.cosmwasm.com/)