// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title IsingJobManager
 * @notice On-chain job registry and proof verification for Nova Ising prover
 * @dev Phase 3 L1 Integration - Thin-state design
 * 
 * Flow:
 * 1. Poster calls postJob() with problem commitment, threshold, reward
 * 2. Solver computes solution off-chain using Nova prover
 * 3. Solver calls submitProof() with compressed Nova proof
 * 4. Contract verifies proof and releases reward
 */
contract IsingJobManager {
    
    //==========================================================================
    // TYPES
    //==========================================================================
    
    enum JobStatus {
        OPEN,       // Accepting solutions
        SOLVED,     // Valid proof submitted
        EXPIRED,    // Deadline passed without solution
        CANCELLED   // Cancelled by poster (before solution)
    }
    
    struct IsingJob {
        bytes32 problemCommitment;  // Poseidon(n_spins, edge_hashes...)
        int64 threshold;            // Energy threshold T (solution must have E ≤ T)
        uint256 reward;             // Reward in wei
        uint256 deadline;           // Block timestamp deadline
        address poster;             // Job poster
        address solver;             // Winning solver (address(0) if unsolved)
        JobStatus status;
        uint256 createdAt;
    }
    
    struct Solution {
        bytes32 spinCommitment;     // Poseidon commitment to spin configuration
        int64 claimedEnergy;        // Claimed energy E
        uint256 proofTimestamp;     // When proof was submitted
    }
    
    //==========================================================================
    // STATE
    //==========================================================================
    
    // Job storage
    mapping(uint256 => IsingJob) public jobs;
    mapping(uint256 => Solution) public solutions;
    uint256 public nextJobId;
    
    // Verifier contract (upgradeable)
    address public verifier;
    address public owner;
    
    // Protocol parameters
    uint256 public minReward = 0.001 ether;
    uint256 public minDeadline = 1 hours;
    uint256 public protocolFeePercent = 1; // 1% fee
    uint256 public collectedFees;
    
    //==========================================================================
    // EVENTS
    //==========================================================================
    
    event JobPosted(
        uint256 indexed jobId,
        address indexed poster,
        bytes32 problemCommitment,
        int64 threshold,
        uint256 reward,
        uint256 deadline
    );
    
    event JobSolved(
        uint256 indexed jobId,
        address indexed solver,
        bytes32 spinCommitment,
        int64 claimedEnergy
    );
    
    event JobCancelled(uint256 indexed jobId);
    event JobExpired(uint256 indexed jobId);
    event VerifierUpdated(address indexed oldVerifier, address indexed newVerifier);
    
    //==========================================================================
    // MODIFIERS
    //==========================================================================
    
    modifier onlyOwner() {
        require(msg.sender == owner, "Not owner");
        _;
    }
    
    //==========================================================================
    // CONSTRUCTOR
    //==========================================================================
    
    constructor(address _verifier) {
        owner = msg.sender;
        verifier = _verifier;
    }
    
    //==========================================================================
    // JOB POSTING
    //==========================================================================
    
    /**
     * @notice Post a new Ising optimization job
     * @param problemCommitment Poseidon hash of the problem (n_spins, edges)
     * @param threshold Energy threshold T - solution must achieve E ≤ T
     * @param deadline Block timestamp by which solution must be submitted
     * @return jobId The ID of the newly created job
     */
    function postJob(
        bytes32 problemCommitment,
        int64 threshold,
        uint256 deadline
    ) external payable returns (uint256 jobId) {
        require(msg.value >= minReward, "Reward too low");
        require(deadline >= block.timestamp + minDeadline, "Deadline too soon");
        require(problemCommitment != bytes32(0), "Invalid commitment");
        
        jobId = nextJobId++;
        
        jobs[jobId] = IsingJob({
            problemCommitment: problemCommitment,
            threshold: threshold,
            reward: msg.value,
            deadline: deadline,
            poster: msg.sender,
            solver: address(0),
            status: JobStatus.OPEN,
            createdAt: block.timestamp
        });
        
        emit JobPosted(
            jobId,
            msg.sender,
            problemCommitment,
            threshold,
            msg.value,
            deadline
        );
    }
    
    //==========================================================================
    // PROOF SUBMISSION
    //==========================================================================
    
    /**
     * @notice Submit a solution proof for an open job
     * @param jobId The job to solve
     * @param spinCommitment Poseidon commitment to the spin configuration
     * @param claimedEnergy The energy achieved (must be ≤ threshold)
     * @param proof The compressed Nova proof bytes
     */
    function submitProof(
        uint256 jobId,
        bytes32 spinCommitment,
        int64 claimedEnergy,
        bytes calldata proof
    ) external {
        IsingJob storage job = jobs[jobId];
        
        require(job.status == JobStatus.OPEN, "Job not open");
        require(block.timestamp <= job.deadline, "Job expired");
        require(claimedEnergy <= job.threshold, "Energy exceeds threshold");
        
        // Verify the Nova proof
        bool valid = _verifyProof(
            job.problemCommitment,
            spinCommitment,
            claimedEnergy,
            job.threshold,
            proof
        );
        require(valid, "Invalid proof");
        
        // Mark job as solved
        job.solver = msg.sender;
        job.status = JobStatus.SOLVED;
        
        // Store solution
        solutions[jobId] = Solution({
            spinCommitment: spinCommitment,
            claimedEnergy: claimedEnergy,
            proofTimestamp: block.timestamp
        });
        
        // Calculate and transfer reward
        uint256 fee = (job.reward * protocolFeePercent) / 100;
        uint256 solverReward = job.reward - fee;
        collectedFees += fee;
        
        (bool success, ) = msg.sender.call{value: solverReward}("");
        require(success, "Transfer failed");
        
        emit JobSolved(jobId, msg.sender, spinCommitment, claimedEnergy);
    }
    
    //==========================================================================
    // PROOF VERIFICATION (Placeholder - requires Nova verifier)
    //==========================================================================
    
    /**
     * @dev Verify a Nova proof. Currently delegates to external verifier.
     * 
     * The proof verifies:
     * 1. Energy was computed correctly for the committed problem
     * 2. claimedEnergy ≤ threshold
     * 3. Spins are binary (0 or 1)
     * 4. Commitments match
     */
    function _verifyProof(
        bytes32 problemCommitment,
        bytes32 spinCommitment,
        int64 claimedEnergy,
        int64 threshold,
        bytes calldata proof
    ) internal view returns (bool) {
        if (verifier == address(0)) {
            // No verifier set - UNSAFE, only for testing
            return proof.length > 0;
        }
        
        // Call external verifier contract
        (bool success, bytes memory result) = verifier.staticcall(
            abi.encodeWithSignature(
                "verify(bytes32,bytes32,int64,int64,bytes)",
                problemCommitment,
                spinCommitment,
                claimedEnergy,
                threshold,
                proof
            )
        );
        
        return success && abi.decode(result, (bool));
    }
    
    //==========================================================================
    // JOB MANAGEMENT
    //==========================================================================
    
    /**
     * @notice Cancel an open job (only poster, before deadline)
     */
    function cancelJob(uint256 jobId) external {
        IsingJob storage job = jobs[jobId];
        
        require(msg.sender == job.poster, "Not poster");
        require(job.status == JobStatus.OPEN, "Job not open");
        
        job.status = JobStatus.CANCELLED;
        
        // Refund reward
        (bool success, ) = job.poster.call{value: job.reward}("");
        require(success, "Refund failed");
        
        emit JobCancelled(jobId);
    }
    
    /**
     * @notice Mark expired jobs and refund poster
     */
    function expireJob(uint256 jobId) external {
        IsingJob storage job = jobs[jobId];
        
        require(job.status == JobStatus.OPEN, "Job not open");
        require(block.timestamp > job.deadline, "Not expired yet");
        
        job.status = JobStatus.EXPIRED;
        
        // Refund reward to poster
        (bool success, ) = job.poster.call{value: job.reward}("");
        require(success, "Refund failed");
        
        emit JobExpired(jobId);
    }
    
    //==========================================================================
    // VIEW FUNCTIONS
    //==========================================================================
    
    function getJob(uint256 jobId) external view returns (IsingJob memory) {
        return jobs[jobId];
    }
    
    function getSolution(uint256 jobId) external view returns (Solution memory) {
        return solutions[jobId];
    }
    
    function getOpenJobs(uint256 start, uint256 count) external view returns (uint256[] memory) {
        uint256[] memory openJobs = new uint256[](count);
        uint256 found = 0;
        
        for (uint256 i = start; i < nextJobId && found < count; i++) {
            if (jobs[i].status == JobStatus.OPEN && block.timestamp <= jobs[i].deadline) {
                openJobs[found++] = i;
            }
        }
        
        // Resize array
        assembly {
            mstore(openJobs, found)
        }
        
        return openJobs;
    }
    
    //==========================================================================
    // ADMIN
    //==========================================================================
    
    function setVerifier(address _verifier) external onlyOwner {
        emit VerifierUpdated(verifier, _verifier);
        verifier = _verifier;
    }
    
    function setProtocolFee(uint256 _feePercent) external onlyOwner {
        require(_feePercent <= 10, "Fee too high");
        protocolFeePercent = _feePercent;
    }
    
    function withdrawFees(address to) external onlyOwner {
        uint256 amount = collectedFees;
        collectedFees = 0;
        (bool success, ) = to.call{value: amount}("");
        require(success, "Withdraw failed");
    }
    
    function transferOwnership(address newOwner) external onlyOwner {
        require(newOwner != address(0), "Invalid owner");
        owner = newOwner;
    }
}
