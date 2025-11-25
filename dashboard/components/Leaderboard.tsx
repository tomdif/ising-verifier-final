'use client'

import useSWR from 'swr'

const fetcher = (url: string) => fetch(url).then(r => r.json())

export default function Leaderboard() {
  const { data: leaderboard, error } = useSWR('/api/leaderboard', fetcher, {
    refreshInterval: 10000
  })

  if (error) return <div className="text-red-500">Failed to load leaderboard</div>
  if (!leaderboard) return <div className="text-gray-500">Loading...</div>

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Top Provers (PUUB Score)</h2>
      
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Rank</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Address</th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">PUUB Score</th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {leaderboard.length === 0 ? (
              <tr>
                <td colSpan={3} className="px-6 py-8 text-center text-gray-500">
                  No provers yet. Be the first!
                </td>
              </tr>
            ) : (
              leaderboard.map((entry: any, idx: number) => (
                <tr key={entry.address} className={idx < 3 ? 'bg-yellow-50' : ''}>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="font-medium text-gray-900">#{entry.rank}</span>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap font-mono text-sm">
                    {entry.address.slice(0, 6)}...{entry.address.slice(-4)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <span className="font-semibold text-indigo-600">
                      {entry.puub_score.toLocaleString()}
                    </span>
                  </td>
                </tr>
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}
