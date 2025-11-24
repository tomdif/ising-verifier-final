# Nova Ising Orchestration Ecosystem

## Overview

Coordinates the decentralized network of Ising problem posters and provers.

## Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                     Orchestration Layer                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Job Index   │    │   Matcher    │    │    PUUB      │       │
│  │  ──────────  │    │   ────────   │    │   ────────   │       │
│  │  • Discovery │───▶│  • Assign    │───▶│  • Track     │       │
│  │  • Filter    │    │  • Auction   │    │  • Score     │       │
│  │  • Priority  │    │  • Reserve   │    │  • Rewards   │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   │                   │                │
│         ▼                   ▼                   ▼                │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Event Bus (WebSocket)                 │    │
│  │  • NewJob • JobClaimed • ProofSubmitted • JobExpired    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
           │                    │                    │
           ▼                    ▼                    ▼
    ┌────────────┐       ┌────────────┐       ┌────────────┐
    │   Poster   │       │   Prover   │       │  L1 Chain  │
    │   Client   │       │   Node     │       │  Contract  │
    └────────────┘       └────────────┘       └────────────┘
```

## Components

### 1. Job Index
- Indexes jobs from L1 contract events
- Provides filtered queries (by size, reward, deadline)
- Caches job metadata for fast access

### 2. Matcher (Job Assignment)
- First-come-first-served or auction-based
- Prover capability matching (GPU, memory)
- Deadline-aware scheduling

### 3. PUUB Accounting
- Proof-of-Useful-Useful-Work tracking
- Prover reputation scores
- Reward distribution metrics

### 4. Event Bus
- Real-time job notifications
- Proof submission broadcasts
- Network status updates

## API Endpoints

### Jobs
- `GET /jobs` - List open jobs
- `GET /jobs/:id` - Job details
- `POST /jobs/:id/claim` - Claim job for proving

### Provers
- `POST /provers/register` - Register as prover
- `GET /provers/:addr/stats` - Prover statistics
- `GET /provers/leaderboard` - Top provers

### PUUB
- `GET /puub/score/:addr` - Get PUUB score
- `GET /puub/history/:addr` - Proof history
