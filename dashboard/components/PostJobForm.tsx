'use client'

import { useState } from 'react'

export default function PostJobForm() {
  const [formData, setFormData] = useState({
    problemFile: null as File | null,
    threshold: '',
    reward: '',
    deadline: '24'
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    alert('Job posting requires wallet connection - coming soon!')
  }

  return (
    <div className="max-w-2xl">
      <h2 className="text-2xl font-bold mb-6">Post New Job</h2>
      
      <form onSubmit={handleSubmit} className="bg-white p-6 rounded-lg shadow space-y-6">
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Problem File (edges)
          </label>
          <input
            type="file"
            accept=".txt,.csv"
            onChange={(e) => setFormData({...formData, problemFile: e.target.files?.[0] || null})}
            className="block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:bg-indigo-50 file:text-indigo-700 hover:file:bg-indigo-100"
          />
          <p className="mt-1 text-sm text-gray-500">
            Format: each line should be: spin_i, spin_j, weight
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Energy Threshold
          </label>
          <input
            type="number"
            value={formData.threshold}
            onChange={(e) => setFormData({...formData, threshold: e.target.value})}
            placeholder="-50000"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
          />
          <p className="mt-1 text-sm text-gray-500">
            Solution must achieve energy ≤ this threshold
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Reward (ETH)
          </label>
          <input
            type="number"
            step="0.001"
            value={formData.reward}
            onChange={(e) => setFormData({...formData, reward: e.target.value})}
            placeholder="0.01"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
          />
          <p className="mt-1 text-sm text-gray-500">
            Minimum: 0.001 ETH
          </p>
        </div>

        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Deadline (hours)
          </label>
          <input
            type="number"
            value={formData.deadline}
            onChange={(e) => setFormData({...formData, deadline: e.target.value})}
            placeholder="24"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-indigo-500 focus:border-indigo-500"
          />
        </div>

        <button
          type="submit"
          className="w-full bg-indigo-600 text-white py-3 px-4 rounded-md hover:bg-indigo-700 transition font-medium"
        >
          Post Job (Connect Wallet)
        </button>
      </form>
    </div>
  )
}
