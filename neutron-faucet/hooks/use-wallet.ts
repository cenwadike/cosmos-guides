"use client"

import { useState, useEffect, useCallback } from "react"

export function useWallet() {
  const [address, setAddress] = useState<string | null>(null)
  const [isConnected, setIsConnected] = useState(false)

  // Check if Keplr is installed
  const isKeplrAvailable = useCallback(() => {
    return typeof window !== "undefined" && "keplr" in window
  }, [])

  // Initialize wallet connection on component mount
  useEffect(() => {
    const checkExistingConnection = async () => {
      if (!isKeplrAvailable()) return

      try {
        // Try to get accounts to see if already connected
        const offlineSigner = window.keplr?.getOfflineSigner("pion-1")
        if (offlineSigner) {
          const accounts = await offlineSigner.getAccounts()
          if (accounts.length > 0) {
            setAddress(accounts[0].address)
            setIsConnected(true)
          }
        }
      } catch (error) {
        console.error("Error checking existing connection:", error)
      }
    }

    checkExistingConnection()
  }, [isKeplrAvailable])

  // Connect wallet
  const connect = useCallback(async () => {
    if (!isKeplrAvailable()) {
      window.open("https://www.keplr.app/", "_blank")
      return
    }

    try {
      // Request connection to Neutron chain
      await window.keplr?.enable("pion-1")

      // Get the offline signer
      const offlineSigner = window.keplr?.getOfflineSigner("pion-1")

      if (offlineSigner) {
        const accounts = await offlineSigner.getAccounts()
        setAddress(accounts[0].address)
        setIsConnected(true)
      }
    } catch (error) {
      console.error("Error connecting wallet:", error)
    }
  }, [isKeplrAvailable])

  // Disconnect wallet
  const disconnect = useCallback(() => {
    setAddress(null)
    setIsConnected(false)
  }, [])

  return {
    address,
    isConnected,
    connect,
    disconnect,
  }
}

