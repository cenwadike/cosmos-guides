"use client"

import { motion } from "framer-motion"
import { Button } from "@/components/ui/button"
import { Wallet, LogOut } from "lucide-react"
import { truncateAddress } from "@/lib/utils"

interface WalletSectionProps {
  address: string | null
  isConnected: boolean
  onConnect: () => void
  onDisconnect: () => void
}

export function WalletSection({ address, isConnected, onConnect, onDisconnect }: WalletSectionProps) {
  return (
    <div className="flex flex-col items-center justify-center p-3 md:p-4 border border-neutral-700 rounded-lg bg-neutral-900/60 backdrop-blur-sm">
      <h2 className="text-lg md:text-xl font-semibold mb-3 md:mb-4 flex items-center">
        <Wallet className="h-4 w-4 mr-2 text-green-500" />
        Wallet Connection
      </h2>

      {isConnected && address ? (
        <motion.div
          className="w-full"
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.3 }}
        >
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 p-3 bg-neutral-800/80 rounded-md border border-neutral-700">
            <div className="flex items-center">
              <div className="w-8 h-8 rounded-full bg-gradient-to-r from-green-500 to-green-600 flex items-center justify-center mr-3">
                <Wallet className="h-4 w-4 text-white" />
              </div>
              <span className="font-mono text-sm md:text-base">{truncateAddress(address)}</span>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={onDisconnect}
              className="text-neutral-400 hover:text-white hover:bg-neutral-700 w-full sm:w-auto"
            >
              <LogOut className="h-4 w-4 mr-2" />
              Disconnect
            </Button>
          </div>
        </motion.div>
      ) : (
        <motion.div
          className="w-full"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ delay: 0.2, duration: 0.5 }}
        >
          <Button
            onClick={onConnect}
            className="w-full bg-gradient-to-r from-green-600 to-green-500 hover:from-green-700 hover:to-green-600 transition-all duration-300"
            whileHover={{ scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
          >
            <Wallet className="h-4 w-4 mr-2" />
            Connect Wallet
          </Button>
        </motion.div>
      )}
    </div>
  )
}

