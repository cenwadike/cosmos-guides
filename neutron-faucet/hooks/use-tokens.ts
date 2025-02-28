"use client"

import { useState, useEffect } from "react"
import type { Token } from "@/types"

export function useTokens() {
  const [tokens, setTokens] = useState<Token[]>([])
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    const fetchTokens = async () => {
      setIsLoading(true)
      try {
        // In a real application, you would fetch this from the contract
        // For now, we'll use mock data
        const mockTokens: Token[] = [
          {
            denom: "untrn",
            symbol: "NTRN",
            name: "Neutron",
            amount: "0.1",
            isNative: true,
          },
          {
            denom: "cw20:token_contract_address_for_tNGN",
            symbol: "tNGN",
            name: "Test Nigerian Naira",
            amount: "100",
            isNative: false,
          },
          {
            denom: "cw20:token_contract_address_for_tATOM",
            symbol: "tATOM",
            name: "Test Cosmos Hub",
            amount: "100",
            isNative: false,
          },
        ]

        // Simulate network delay
        setTimeout(() => {
          setTokens(mockTokens)
          setIsLoading(false)
        }, 1000)
      } catch (error) {
        console.error("Error fetching tokens:", error)
        setIsLoading(false)
      }
    }

    fetchTokens()
  }, [])

  return { tokens, isLoading }
}

