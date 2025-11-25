// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Script.sol";
import "../src/IsingJobManager.sol";
import "../src/NovaVerifier.sol";

contract DeployIsing is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        
        vm.startBroadcast(deployerPrivateKey);
        
        // 1. Deploy NovaVerifier first
        NovaVerifier verifier = new NovaVerifier();
        console.log("NovaVerifier deployed at:", address(verifier));
        
        // 2. Deploy IsingJobManager with verifier address
        IsingJobManager manager = new IsingJobManager(address(verifier));
        console.log("IsingJobManager deployed at:", address(manager));
        
        // 3. Log configuration
        console.log("");
        console.log("=== Deployment Summary ===");
        console.log("Network: Sepolia");
        console.log("Verifier:", address(verifier));
        console.log("JobManager:", address(manager));
        console.log("Owner:", manager.owner());
        console.log("Min Reward:", manager.minReward());
        console.log("Protocol Fee:", manager.protocolFeePercent(), "%");
        
        vm.stopBroadcast();
    }
}
