"use client"

import { useState, useCallback } from "react"
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { GasPrice } from "@cosmjs/stargate"
import { OfflineSigner } from "@cosmjs/proto-signing" // Add this import

// Extend Window interface for Keplr
declare global {
  interface Window {
    keplr?: {
      enable: (chainId: string) => Promise<void>
      getOfflineSigner: (chainId: string) => OfflineSigner
      getKey: (chainId: string) => Promise<{
        bech32Address: string
        pubKey: Uint8Array
        name: string
        isNanoLedger: boolean
      }>
    }
  }
}

const RPC_ENDPOINT = "https://rpc-palvus.pion-1.ntrn.tech"
const CONTRACT_ADDRESS = "neutron1w0ls4rscaug5gz30envteezu2yfug2ychs3ts204rzmrfr66g7dss2qlt7"
const CHAIN_ID = "pion-1"

// Define the ExecuteMsg interface
interface ExecuteMsg {
  claim: {}
}

export function useClaim() {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  const executeClaim = async (account: string): Promise<void> => {
    if (!window.keplr) {
      throw new Error("Please install Keplr wallet extension")
    }

    try {
      // Enable Keplr for the chain
      await window.keplr.enable(CHAIN_ID)

      // Get offline signer
      const offlineSigner = window.keplr.getOfflineSigner(CHAIN_ID)

      // Create CosmWasm client
      const client = await SigningCosmWasmClient.connectWithSigner(
        RPC_ENDPOINT,
        offlineSigner,
        { gasPrice: GasPrice.fromString("0.025untrn") }
      )

      // Construct claim message
      const msg: ExecuteMsg = {
        claim: {}
      }

      // Execute transaction
      const result = await client.execute(
        account,
        CONTRACT_ADDRESS,
        msg,
        "auto",
        "Claiming tokens"
      )

      console.log("Claim successful:", result)
      setSuccess(true)
    } catch (err) {
      throw err instanceof Error ? err : new Error("Unknown error occurred")
    }
  }

  const claim = useCallback(async (address?: string) => {
    setIsLoading(true)
    setError(null)
    setSuccess(false)

    try {
      if (!window.keplr) {
        throw new Error("Keplr wallet not found")
      }

      // Get account info if not provided
      const key = await window.keplr.getKey(CHAIN_ID)
      const account = address || key.bech32Address

      await executeClaim(account)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to claim tokens"
      setError(errorMessage)
      console.error("Claim error:", err)
    } finally {
      setIsLoading(false)
    }
  }, [])

  const resetStatus = useCallback(() => {
    setError(null)
    setSuccess(false)
  }, [])

  return {
    claim,
    isLoading,
    error,
    success,
    resetStatus,
  }
}