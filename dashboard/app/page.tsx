'use client'

import { useState, useEffect } from 'react'
import JobList from '@/components/JobList'
import PostJobForm from '@/components/PostJobForm'
import Leaderboard from '@/components/Leaderboard'

export default function Home() {
  const [activeTab, setActiveTab] = useState<'jobs' | 'post' | 'leaderboard'>('jobs')

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow">
        <div className="max-w-7xl mx-auto px-4 py-6">
          <h1 className="text-3xl font-bold text-gray-900">
            Nova Ising Prover Dashboard
          </h1>
          <p className="mt-2 text-gray-600">
            Decentralized Ising optimization marketplace
          </p>
        </div>
      </header>

      {/* Navigation */}
      <nav className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4">
          <div className="flex space-x-8">
            <button
              onClick={() => setActiveTab('jobs')}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'jobs'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Open Jobs
            </button>
            <button
              onClick={() => setActiveTab('post')}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'post'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Post Job
            </button>
            <button
              onClick={() => setActiveTab('leaderboard')}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === 'leaderboard'
                  ? 'border-indigo-500 text-indigo-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              Leaderboard
            </button>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 py-8">
        {activeTab === 'jobs' && <JobList />}
        {activeTab === 'post' && <PostJobForm />}
        {activeTab === 'leaderboard' && <Leaderboard />}
      </main>

      {/* Footer */}
      <footer className="bg-white border-t mt-12">
        <div className="max-w-7xl mx-auto px-4 py-6 text-center text-gray-500">
          <p>Nova Ising Prover | Quantum-Resistant Optimization</p>
        </div>
      </footer>
    </div>
  )
}
