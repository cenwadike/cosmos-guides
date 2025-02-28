"use client"

import { motion } from "framer-motion"
import { Button } from "@/components/ui/button"
import { Loader2 } from "lucide-react"

interface ClaimButtonProps {
  onClick: () => void
  disabled: boolean
  isLoading: boolean
}

export function ClaimButton({ onClick, disabled, isLoading }: ClaimButtonProps) {
  return (
    <motion.div whileHover={{ scale: disabled ? 1 : 1.02 }} whileTap={{ scale: disabled ? 1 : 0.98 }}>
      <Button
        onClick={onClick}
        disabled={disabled}
        className="w-full h-12 text-lg bg-gradient-to-r from-green-600 to-green-500 hover:from-green-700 hover:to-green-600 disabled:opacity-50 transition-all duration-300"
      >
        {isLoading ? (
          <>
            <Loader2 className="mr-2 h-5 w-5 animate-spin" />
            Claiming...
          </>
        ) : (
          "Claim Tokens"
        )}
      </Button>
    </motion.div>
  )
}

