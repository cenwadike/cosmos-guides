import { FaucetContainer } from "@/components/faucet-container"

export default function Home() {
  return (
    <main className="min-h-screen bg-gradient-to-b from-neutral-900 to-black text-white overflow-hidden">
      <div className="container mx-auto px-4 py-8 md:py-12 max-w-3xl">
        <div className="space-y-2 mb-8 md:mb-12">
          <h1 className="text-3xl md:text-5xl font-bold text-center bg-clip-text text-transparent bg-gradient-to-r from-green-400 to-green-600">
            Naija Hackatom Faucet
          </h1>
          <p className="text-center text-neutral-400 text-sm md:text-base">
            Claim testnet tokens for the Neutron blockchain
          </p>
        </div>
        <FaucetContainer />
      </div>
    </main>
  )
}

