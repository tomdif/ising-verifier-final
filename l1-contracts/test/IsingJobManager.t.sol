// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/IsingJobManager.sol";
import "../src/NovaVerifier.sol";

contract IsingJobManagerTest is Test {
    IsingJobManager public manager;
    NovaVerifier public verifier;
    
    address public owner = address(this);
    address public poster = address(0x1);
    address public solver = address(0x2);
    address public other = address(0x3);
    
    bytes32 public problemCommitment = keccak256("test_problem");
    bytes32 public spinCommitment = keccak256("test_spins");
    int64 public threshold = -50000;
    
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
    
    function setUp() public {
        verifier = new NovaVerifier();
        manager = new IsingJobManager(address(verifier));
        
        // Fund test accounts
        vm.deal(poster, 100 ether);
        vm.deal(solver, 10 ether);
        vm.deal(other, 10 ether);
    }
    
    //==========================================================================
    // JOB POSTING TESTS
    //==========================================================================
    
    function test_PostJob() public {
        uint256 reward = 1 ether;
        uint256 deadline = block.timestamp + 1 days;
        
        vm.prank(poster);
        vm.expectEmit(true, true, false, true);
        emit JobPosted(0, poster, problemCommitment, threshold, reward, deadline);
        
        uint256 jobId = manager.postJob{value: reward}(
            problemCommitment,
            threshold,
            deadline
        );
        
        assertEq(jobId, 0);
        
        IsingJobManager.IsingJob memory job = manager.getJob(jobId);
        assertEq(job.problemCommitment, problemCommitment);
        assertEq(job.threshold, threshold);
        assertEq(job.reward, reward);
        assertEq(job.deadline, deadline);
        assertEq(job.poster, poster);
        assertEq(job.solver, address(0));
        assertEq(uint(job.status), uint(IsingJobManager.JobStatus.OPEN));
    }
    
    function test_PostJob_RevertIfRewardTooLow() public {
        vm.prank(poster);
        vm.expectRevert("Reward too low");
        manager.postJob{value: 0.0001 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
    }
    
    function test_PostJob_RevertIfDeadlineTooSoon() public {
        vm.prank(poster);
        vm.expectRevert("Deadline too soon");
        manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 30 minutes // Less than minDeadline (1 hour)
        );
    }
    
    function test_PostJob_RevertIfInvalidCommitment() public {
        vm.prank(poster);
        vm.expectRevert("Invalid commitment");
        manager.postJob{value: 1 ether}(
            bytes32(0),
            threshold,
            block.timestamp + 1 days
        );
    }
    
    function test_PostMultipleJobs() public {
        vm.startPrank(poster);
        
        uint256 job0 = manager.postJob{value: 1 ether}(
            keccak256("problem1"),
            -10000,
            block.timestamp + 1 days
        );
        
        uint256 job1 = manager.postJob{value: 2 ether}(
            keccak256("problem2"),
            -20000,
            block.timestamp + 2 days
        );
        
        vm.stopPrank();
        
        assertEq(job0, 0);
        assertEq(job1, 1);
        assertEq(manager.nextJobId(), 2);
    }
    
    //==========================================================================
    // PROOF SUBMISSION TESTS
    //==========================================================================
    
    function test_SubmitProof() public {
        // Post job
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        // Create valid proof (stub verifier accepts any non-empty proof)
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        int64 claimedEnergy = threshold - 1000; // Below threshold
        
        uint256 solverBalanceBefore = solver.balance;
        
        vm.prank(solver);
        vm.expectEmit(true, true, false, true);
        emit JobSolved(jobId, solver, spinCommitment, claimedEnergy);
        
        manager.submitProof(jobId, spinCommitment, claimedEnergy, proof);
        
        // Check job status
        IsingJobManager.IsingJob memory job = manager.getJob(jobId);
        assertEq(uint(job.status), uint(IsingJobManager.JobStatus.SOLVED));
        assertEq(job.solver, solver);
        
        // Check solution stored
        IsingJobManager.Solution memory solution = manager.getSolution(jobId);
        assertEq(solution.spinCommitment, spinCommitment);
        assertEq(solution.claimedEnergy, claimedEnergy);
        
        // Check reward transferred (minus 1% fee)
        uint256 expectedReward = 1 ether * 99 / 100;
        assertEq(solver.balance, solverBalanceBefore + expectedReward);
    }
    
    function test_SubmitProof_RevertIfJobNotOpen() public {
        // Post and solve job
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        vm.prank(solver);
        manager.submitProof(jobId, spinCommitment, threshold - 1000, proof);
        
        // Try to submit again
        vm.prank(other);
        vm.expectRevert("Job not open");
        manager.submitProof(jobId, spinCommitment, threshold - 2000, proof);
    }
    
    function test_SubmitProof_RevertIfExpired() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        // Fast forward past deadline
        vm.warp(block.timestamp + 2 days);
        
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        vm.prank(solver);
        vm.expectRevert("Job expired");
        manager.submitProof(jobId, spinCommitment, threshold - 1000, proof);
    }
    
    function test_SubmitProof_RevertIfEnergyExceedsThreshold() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        vm.prank(solver);
        vm.expectRevert("Energy exceeds threshold");
        manager.submitProof(jobId, spinCommitment, threshold + 1000, proof); // Above threshold
    }
    
    function test_SubmitProof_RevertIfInvalidProof() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        // Empty proof should fail (stub verifier requires length >= 32)
        bytes memory proof = new bytes(10);
        
        vm.prank(solver);
        vm.expectRevert("Invalid proof");
        manager.submitProof(jobId, spinCommitment, threshold - 1000, proof);
    }
    
    //==========================================================================
    // JOB CANCELLATION TESTS
    //==========================================================================
    
    function test_CancelJob() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        uint256 posterBalanceBefore = poster.balance;
        
        vm.prank(poster);
        vm.expectEmit(true, false, false, false);
        emit JobCancelled(jobId);
        manager.cancelJob(jobId);
        
        IsingJobManager.IsingJob memory job = manager.getJob(jobId);
        assertEq(uint(job.status), uint(IsingJobManager.JobStatus.CANCELLED));
        assertEq(poster.balance, posterBalanceBefore + 1 ether);
    }
    
    function test_CancelJob_RevertIfNotPoster() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        vm.prank(other);
        vm.expectRevert("Not poster");
        manager.cancelJob(jobId);
    }
    
    function test_CancelJob_RevertIfAlreadySolved() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        vm.prank(solver);
        manager.submitProof(jobId, spinCommitment, threshold - 1000, proof);
        
        vm.prank(poster);
        vm.expectRevert("Job not open");
        manager.cancelJob(jobId);
    }
    
    //==========================================================================
    // JOB EXPIRATION TESTS
    //==========================================================================
    
    function test_ExpireJob() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        uint256 posterBalanceBefore = poster.balance;
        
        // Fast forward past deadline
        vm.warp(block.timestamp + 2 days);
        
        vm.expectEmit(true, false, false, false);
        emit JobExpired(jobId);
        manager.expireJob(jobId);
        
        IsingJobManager.IsingJob memory job = manager.getJob(jobId);
        assertEq(uint(job.status), uint(IsingJobManager.JobStatus.EXPIRED));
        assertEq(poster.balance, posterBalanceBefore + 1 ether);
    }
    
    function test_ExpireJob_RevertIfNotExpired() public {
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        vm.expectRevert("Not expired yet");
        manager.expireJob(jobId);
    }
    
    //==========================================================================
    // VIEW FUNCTION TESTS
    //==========================================================================
    
    function test_GetOpenJobs() public {
        // Post several jobs
        vm.startPrank(poster);
        manager.postJob{value: 1 ether}(keccak256("p1"), -10000, block.timestamp + 1 days);
        manager.postJob{value: 1 ether}(keccak256("p2"), -20000, block.timestamp + 1 days);
        manager.postJob{value: 1 ether}(keccak256("p3"), -30000, block.timestamp + 1 days);
        vm.stopPrank();
        
        uint256[] memory openJobs = manager.getOpenJobs(0, 10);
        assertEq(openJobs.length, 3);
        assertEq(openJobs[0], 0);
        assertEq(openJobs[1], 1);
        assertEq(openJobs[2], 2);
    }
    
    //==========================================================================
    // ADMIN TESTS
    //==========================================================================
    
    function test_SetVerifier() public {
        address newVerifier = address(0x999);
        manager.setVerifier(newVerifier);
        assertEq(manager.verifier(), newVerifier);
    }
    
    function test_SetVerifier_RevertIfNotOwner() public {
        vm.prank(other);
        vm.expectRevert("Not owner");
        manager.setVerifier(address(0x999));
    }
    
    function test_SetProtocolFee() public {
        manager.setProtocolFee(5);
        assertEq(manager.protocolFeePercent(), 5);
    }
    
    function test_SetProtocolFee_RevertIfTooHigh() public {
        vm.expectRevert("Fee too high");
        manager.setProtocolFee(15);
    }
    
    function test_WithdrawFees() public {
        // Post and solve job to generate fees
        vm.prank(poster);
        uint256 jobId = manager.postJob{value: 1 ether}(
            problemCommitment,
            threshold,
            block.timestamp + 1 days
        );
        
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        vm.prank(solver);
        manager.submitProof(jobId, spinCommitment, threshold - 1000, proof);
        
        uint256 expectedFees = 1 ether / 100; // 1%
        assertEq(manager.collectedFees(), expectedFees);
        
        address feeRecipient = address(0x888);
        uint256 balanceBefore = feeRecipient.balance;
        
        manager.withdrawFees(feeRecipient);
        
        assertEq(manager.collectedFees(), 0);
        assertEq(feeRecipient.balance, balanceBefore + expectedFees);
    }
    
    function test_TransferOwnership() public {
        address newOwner = address(0x777);
        manager.transferOwnership(newOwner);
        assertEq(manager.owner(), newOwner);
    }
}
