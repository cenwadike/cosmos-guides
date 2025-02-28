"use client"

import { useState, useEffect } from "react"
import { motion } from "framer-motion"
import { WalletSection } from "@/components/wallet-section"
import { TokenList } from "@/components/token-list"
import { ClaimButton } from "@/components/claim-button"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { AlertCircle, CheckCircle2 } from "lucide-react"
import { useWallet } from "@/hooks/use-wallet"
import { useTokens } from "@/hooks/use-tokens"
import { useClaim } from "@/hooks/use-claim"

export function FaucetContainer() {
  const { address, isConnected, connect, disconnect } = useWallet()
  const { tokens, isLoading: isLoadingTokens } = useTokens()
  const { claim, isLoading: isClaiming, error, success, resetStatus } = useClaim()
  const [timeRemaining, setTimeRemaining] = useState<number | null>(null)

  useEffect(() => {
    if (error && error.includes("Rate limit exceeded")) {
      // Extract seconds from error message: "Rate limit exceeded. You can claim again in X seconds"
      const match = error.match(/in (\d+) seconds/)
      if (match && match[1]) {
        setTimeRemaining(Number.parseInt(match[1]))
      }
    } else {
      setTimeRemaining(null)
    }
  }, [error])

  useEffect(() => {
    let timer: NodeJS.Timeout
    if (timeRemaining && timeRemaining > 0) {
      timer = setTimeout(() => {
        setTimeRemaining((prev) => (prev ? prev - 1 : null))
      }, 1000)
    }
    return () => clearTimeout(timer)
  }, [timeRemaining])

  useEffect(() => {
    if (success) {
      const timer = setTimeout(() => {
        resetStatus()
      }, 5000)
      return () => clearTimeout(timer)
    }
  }, [success, resetStatus])

  const handleClaim = async () => {
    if (!isConnected || !address) return
    await claim(address)
  }

  const formatTime = (seconds: number): string => {
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60

    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`
    } else {
      return `${secs}s`
    }
  }

  const containerVariants = {
    hidden: { opacity: 0, y: 20 },
    visible: {
      opacity: 1,
      y: 0,
      transition: {
        duration: 0.6,
        ease: "easeOut",
      },
    },
  }

  const contentVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        delay: 0.3,
        duration: 0.5,
        ease: "easeOut",
      },
    },
  }

  return (
    <motion.div
      className="bg-neutral-800/80 backdrop-blur-sm rounded-xl p-4 md:p-6 shadow-lg border border-neutral-700"
      initial="hidden"
      animate="visible"
      variants={containerVariants}
    >
      <WalletSection address={address} isConnected={isConnected} onConnect={connect} onDisconnect={disconnect} />

      {isConnected && (
        <motion.div initial="hidden" animate="visible" variants={contentVariants}>
          <div className="mt-6 md:mt-8">
            <h2 className="text-xl font-semibold mb-3 md:mb-4 flex items-center">
              <span className="inline-block w-2 h-2 rounded-full bg-green-500 mr-2 animate-pulse"></span>
              Available Tokens
            </h2>
            <TokenList tokens={tokens} isLoading={isLoadingTokens} />
          </div>

          <div className="mt-6 md:mt-8">
            {error && !timeRemaining && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
                <Alert variant="destructive" className="mb-4">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              </motion.div>
            )}

            {timeRemaining && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
                <Alert variant="destructive" className="mb-4">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>
                    Rate limit exceeded. You can claim again in {formatTime(timeRemaining)}
                  </AlertDescription>
                </Alert>
              </motion.div>
            )}

            {success && (
              <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.3 }}>
                <Alert className="mb-4 bg-green-900/60 backdrop-blur-sm border-green-800 text-green-100">
                  <CheckCircle2 className="h-4 w-4" />
                  <AlertDescription>Tokens claimed successfully!</AlertDescription>
                </Alert>
              </motion.div>
            )}

            <ClaimButton onClick={handleClaim} disabled={isClaiming || !!timeRemaining} isLoading={isClaiming} />
          </div>
        </motion.div>
      )}
    </motion.div>
  )
}

