'use client'

import useSWR from 'swr'

const fetcher = (url: string) => fetch(url).then(r => r.json())

export default function JobList() {
  const { data: jobs, error } = useSWR('/api/jobs', fetcher, {
    refreshInterval: 5000
  })

  if (error) return <div className="text-red-500">Failed to load jobs</div>
  if (!jobs) return <div className="text-gray-500">Loading...</div>

  return (
    <div className="space-y-4">
      <h2 className="text-2xl font-bold">Open Jobs</h2>
      
      {jobs.length === 0 ? (
        <div className="bg-white p-8 rounded-lg shadow text-center text-gray-500">
          No open jobs available. Be the first to post one!
        </div>
      ) : (
        <div className="grid gap-4">
          {jobs.map((job: any) => (
            <div key={job.id} className="bg-white p-6 rounded-lg shadow hover:shadow-lg transition">
              <div className="flex justify-between items-start">
                <div>
                  <h3 className="text-lg font-semibold text-gray-900">
                    Job #{job.id}
                  </h3>
                  <p className="text-sm text-gray-500 mt-1">
                    Problem: {job.problem_commitment.slice(0, 10)}...{job.problem_commitment.slice(-8)}
                  </p>
                </div>
                <span className="px-3 py-1 bg-green-100 text-green-800 rounded-full text-sm">
                  Open
                </span>
              </div>
              
              <div className="mt-4 grid grid-cols-2 gap-4">
                <div>
                  <p className="text-sm text-gray-500">Spins</p>
                  <p className="font-medium">{job.n_spins?.toLocaleString() || 'N/A'}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Edges</p>
                  <p className="font-medium">{job.n_edges?.toLocaleString() || 'N/A'}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Threshold</p>
                  <p className="font-medium">{job.threshold}</p>
                </div>
                <div>
                  <p className="text-sm text-gray-500">Reward</p>
                  <p className="font-medium">{(parseInt(job.reward_wei) / 1e18).toFixed(4)} ETH</p>
                </div>
              </div>

              <button className="mt-4 w-full bg-indigo-600 text-white py-2 px-4 rounded hover:bg-indigo-700 transition">
                Claim Job
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
