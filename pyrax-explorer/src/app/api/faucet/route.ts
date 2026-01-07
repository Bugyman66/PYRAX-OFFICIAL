import { NextResponse } from 'next/server'

export const dynamic = 'force-dynamic'

// In-memory rate limiting (for demo - use Redis in production)
const rateLimitMap = new Map<string, number>()
const COOLDOWN_MS = 60 * 60 * 1000 // 1 hour

export async function POST(request: Request) {
  try {
    const body = await request.json()
    const { address } = body

    if (!address || typeof address !== 'string') {
      return NextResponse.json(
        { error: 'Valid address is required' },
        { status: 400 }
      )
    }

    // Validate address format (basic check)
    if (!address.match(/^0x[a-fA-F0-9]{40}$/) && !address.match(/^[a-zA-Z0-9]{32,64}$/)) {
      return NextResponse.json(
        { error: 'Invalid address format' },
        { status: 400 }
      )
    }

    // Check rate limit
    const lastRequest = rateLimitMap.get(address.toLowerCase())
    const now = Date.now()
    
    if (lastRequest && now - lastRequest < COOLDOWN_MS) {
      const remainingMs = COOLDOWN_MS - (now - lastRequest)
      const remainingMins = Math.ceil(remainingMs / 60000)
      return NextResponse.json(
        { error: `Please wait ${remainingMins} minutes before requesting again` },
        { status: 429 }
      )
    }

    // TODO: Connect to actual faucet service
    // For now, simulate a successful faucet request
    const faucetUrl = process.env.FAUCET_URL || 'http://localhost:8081'
    
    try {
      const response = await fetch(`${faucetUrl}/request`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ address }),
      })

      if (response.ok) {
        const data = await response.json()
        rateLimitMap.set(address.toLowerCase(), now)
        return NextResponse.json({
          success: true,
          amount: data.amount || 100,
          txHash: data.txHash,
        })
      }
    } catch (error) {
      // Faucet service not available, return demo response
      console.log('Faucet service not available, returning demo response')
    }

    // Demo response when faucet service is not available
    rateLimitMap.set(address.toLowerCase(), now)
    const demoTxHash = `0x${Array.from({ length: 64 }, () => 
      Math.floor(Math.random() * 16).toString(16)
    ).join('')}`

    return NextResponse.json({
      success: true,
      amount: 100,
      txHash: demoTxHash,
      demo: true,
    })

  } catch (error) {
    console.error('Faucet error:', error)
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    )
  }
}
