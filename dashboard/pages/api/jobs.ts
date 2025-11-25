import type { NextApiRequest, NextApiResponse } from 'next'

// Mock data - in production, fetch from orchestrator API
const mockJobs = [
  {
    id: 0,
    problem_commitment: '0x7caae4a1d969553eae05f207d7497329ee9bbf81c29a01303b19c41775ef2d1e',
    threshold: -50000,
    reward_wei: '10000000000000000', // 0.01 ETH
    n_spins: 10000,
    n_edges: 60000,
    status: 'Open'
  }
]

export default function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method === 'GET') {
    res.status(200).json(mockJobs)
  } else {
    res.status(405).json({ error: 'Method not allowed' })
  }
}
