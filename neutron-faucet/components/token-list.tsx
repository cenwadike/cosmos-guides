"use client"

import { motion } from "framer-motion"
import { Skeleton } from "@/components/ui/skeleton"
import type { Token } from "@/types"

interface TokenListProps {
  tokens: Token[]
  isLoading: boolean
}

export function TokenList({ tokens, isLoading }: TokenListProps) {
  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.1,
      },
    },
  }

  const itemVariants = {
    hidden: { opacity: 0, y: 10 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.3 },
    },
  }

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <div key={i} className="flex items-center p-3 border border-neutral-700 rounded-md bg-neutral-900/60">
            <Skeleton className="h-10 w-10 rounded-full mr-3" />
            <div className="space-y-2">
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-3 w-16" />
            </div>
            <Skeleton className="h-5 w-20 ml-auto" />
          </div>
        ))}
      </div>
    )
  }

  return (
    <motion.div className="space-y-3" variants={containerVariants} initial="hidden" animate="visible">
      {tokens.map((token, index) => (
        <motion.div
          key={token.denom}
          className="flex items-center p-3 border border-neutral-700 rounded-md bg-neutral-900/60 backdrop-blur-sm hover:bg-neutral-800 transition-colors duration-200"
          variants={itemVariants}
          whileHover={{ scale: 1.01 }}
          transition={{ duration: 0.2 }}
        >
          <div className="w-10 h-10 rounded-full bg-gradient-to-r from-green-500 to-green-600 flex items-center justify-center mr-3">
            <span className="text-sm font-bold">{token.symbol.charAt(0)}</span>
          </div>
          <div>
            <div className="font-medium">{token.symbol}</div>
            <div className="text-sm text-neutral-400">{token.name}</div>
          </div>
          <div className="ml-auto font-mono text-sm">
            {token.amount} {token.symbol}
          </div>
        </motion.div>
      ))}
    </motion.div>
  )
}

