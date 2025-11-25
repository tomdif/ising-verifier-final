#!/bin/bash
# Sepolia Interaction Helper

set -e

# Load environment
source .env

MANAGER=$ISING_JOB_MANAGER_ADDRESS
RPC=$SEPOLIA_RPC_URL

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

case "$1" in
  "status")
    echo -e "${BLUE}=== Contract Status ===${NC}"
    echo "Manager: $MANAGER"
    echo ""
    echo -n "Min Reward: "
    cast call $MANAGER "minReward()(uint256)" --rpc-url $RPC | xargs -I {} echo "{} wei"
    echo -n "Protocol Fee: "
    cast call $MANAGER "protocolFeePercent()(uint256)" --rpc-url $RPC | xargs -I {} echo "{}%"
    echo -n "Next Job ID: "
    cast call $MANAGER "nextJobId()(uint256)" --rpc-url $RPC
    echo -n "Verifier: "
    cast call $MANAGER "verifier()(address)" --rpc-url $RPC
    ;;

  "post-job")
    PROBLEM=$2
    THRESHOLD=$3
    REWARD=$4
    DEADLINE=$(echo "$(cast block latest timestamp --rpc-url $RPC) + 86400" | bc)
    
    echo -e "${BLUE}=== Posting Job ===${NC}"
    echo "Problem: $PROBLEM"
    echo "Threshold: $THRESHOLD"
    echo "Reward: $REWARD"
    echo "Deadline: $DEADLINE"
    
    cast send $MANAGER \
      "postJob(bytes32,int64,uint256)(uint256)" \
      $PROBLEM $THRESHOLD $DEADLINE \
      --value $REWARD \
      --private-key $PRIVATE_KEY \
      --rpc-url $RPC
    
    echo -e "${GREEN}Job posted!${NC}"
    ;;

  "get-job")
    JOB_ID=$2
    echo -e "${BLUE}=== Job $JOB_ID ===${NC}"
    cast call $MANAGER "getJob(uint256)" $JOB_ID --rpc-url $RPC
    ;;

  "open-jobs")
    echo -e "${BLUE}=== Open Jobs ===${NC}"
    cast call $MANAGER "getOpenJobs(uint256,uint256)" 0 10 --rpc-url $RPC
    ;;

  *)
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  status           - Show contract status"
    echo "  post-job <commitment> <threshold> <reward>"
    echo "  get-job <id>     - Get job details"
    echo "  open-jobs        - List open jobs"
    ;;
esac
