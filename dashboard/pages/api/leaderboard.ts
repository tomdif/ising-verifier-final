import type { NextApiRequest, NextApiResponse } from 'next'

// Mock data - in production, fetch from orchestrator API
const mockLeaderboard = [
  {
    rank: 1,
    address: '0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb',
    puub_score: 150000
  },
  {
    rank: 2,
    address: '0x5A0b54D5dc17e0AaD3Ac362E7dC20888e6FcEb',
    puub_score: 120000
  }
]

export default function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method === 'GET') {
    res.status(200).json(mockLeaderboard)
  } else {
    res.status(405).json({ error: 'Method not allowed' })
  }
}
