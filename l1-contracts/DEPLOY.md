# Sepolia Testnet Deployment

## Prerequisites

1. **Install Foundry**
```bash
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
```

2. **Get Sepolia ETH**
   - Alchemy Faucet: https://sepoliafaucet.com/
   - Infura Faucet: https://www.infura.io/faucet/sepolia
   - Need ~0.1 ETH for deployment

3. **Get RPC URL**
   - Alchemy: https://www.alchemy.com/
   - Infura: https://infura.io/
   - Or use public: `https://rpc.sepolia.org`

4. **Get Etherscan API Key** (for verification)
   - https://etherscan.io/apis

## Setup

1. Copy environment template:
```bash
   cp .env.example .env
```

2. Edit `.env` with your values:
```
   PRIVATE_KEY=your_private_key_without_0x
   SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY
   ETHERSCAN_API_KEY=your_etherscan_key
```

3. Install dependencies:
```bash
   forge install foundry-rs/forge-std
```

## Deploy
```bash
# Load environment
source .env

# Deploy to Sepolia
forge script script/Deploy.s.sol:DeployIsing \
  --rpc-url $SEPOLIA_RPC_URL \
  --broadcast \
  --verify \
  -vvvv
```

## Verify Contracts (if auto-verify fails)
```bash
# Verify NovaVerifier
forge verify-contract \
  --chain sepolia \
  --etherscan-api-key $ETHERSCAN_API_KEY \
  <VERIFIER_ADDRESS> \
  src/NovaVerifier.sol:NovaVerifier

# Verify IsingJobManager
forge verify-contract \
  --chain sepolia \
  --etherscan-api-key $ETHERSCAN_API_KEY \
  --constructor-args $(cast abi-encode "constructor(address)" <VERIFIER_ADDRESS>) \
  <MANAGER_ADDRESS> \
  src/IsingJobManager.sol:IsingJobManager
```

## Post-Deployment

After deployment, save the addresses:
```bash
# Add to your config
NOVA_VERIFIER_ADDRESS=0x...
ISING_JOB_MANAGER_ADDRESS=0x...
```

## Test Interaction
```bash
# Check contract state
cast call $ISING_JOB_MANAGER_ADDRESS "minReward()" --rpc-url $SEPOLIA_RPC_URL
cast call $ISING_JOB_MANAGER_ADDRESS "verifier()" --rpc-url $SEPOLIA_RPC_URL

# Post a test job (requires ETH)
cast send $ISING_JOB_MANAGER_ADDRESS \
  "postJob(bytes32,int64,uint256)" \
  0xabababababababababababababababababababababababababababababababab \
  -50000 \
  $(cast block latest timestamp --rpc-url $SEPOLIA_RPC_URL | xargs -I {} echo "{}+86400" | bc) \
  --value 0.01ether \
  --private-key $PRIVATE_KEY \
  --rpc-url $SEPOLIA_RPC_URL
```

## Contract Addresses (after deployment)

| Contract | Address | Etherscan |
|----------|---------|-----------|
| NovaVerifier | `TBD` | [View](https://sepolia.etherscan.io/address/TBD) |
| IsingJobManager | `TBD` | [View](https://sepolia.etherscan.io/address/TBD) |

## Network Info

- **Chain ID**: 11155111
- **Explorer**: https://sepolia.etherscan.io/
- **Faucets**: See prerequisites above

## Deployment Status

**Contracts Ready** ✅
- All tests passing (33/33)
- Deployment script tested
- Estimated gas: ~0.003 ETH

**Pending:**
- Testnet ETH for deployment

**Your Deployer Address:**
```
0xC625656b370eEF2C5870ddb910FeF21aBE94B207
```

Get Sepolia ETH from:
- https://sepoliafaucet.com/
- https://cloud.google.com/application/web3/faucet/ethereum/sepolia

Once you have ~0.01 ETH, run:
```bash
source .env
forge script script/Deploy.s.sol:DeployIsing --rpc-url $SEPOLIA_RPC_URL --broadcast
```
