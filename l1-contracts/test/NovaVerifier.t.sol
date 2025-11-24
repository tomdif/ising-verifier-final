// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/NovaVerifier.sol";

contract NovaVerifierTest is Test {
    NovaVerifier public verifier;
    
    bytes32 public problemCommitment = keccak256("test_problem");
    bytes32 public spinCommitment = keccak256("test_spins");
    int64 public threshold = -50000;
    int64 public claimedEnergy = -60000; // Below threshold
    
    function setUp() public {
        verifier = new NovaVerifier();
    }
    
    //==========================================================================
    // STUB MODE TESTS
    //==========================================================================
    
    function test_StubMode_AcceptsValidProof() public {
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        bool valid = verifier.verify(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            threshold,
            proof
        );
        
        assertTrue(valid);
    }
    
    function test_StubMode_RejectsShortProof() public {
        bytes memory proof = new bytes(10); // Too short
        
        bool valid = verifier.verify(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            threshold,
            proof
        );
        
        assertFalse(valid);
    }
    
    function test_StubMode_RejectsEnergyAboveThreshold() public {
        bytes memory proof = new bytes(100);
        proof[0] = 0x01;
        
        int64 badEnergy = threshold + 1000; // Above threshold
        
        bool valid = verifier.verify(
            problemCommitment,
            spinCommitment,
            badEnergy,
            threshold,
            proof
        );
        
        assertFalse(valid);
    }
    
    //==========================================================================
    // MODE SWITCHING TESTS
    //==========================================================================
    
    function test_SetMode() public {
        assertEq(uint(verifier.mode()), uint(NovaVerifier.Mode.STUB));
        
        verifier.setMode(NovaVerifier.Mode.OPTIMISTIC);
        assertEq(uint(verifier.mode()), uint(NovaVerifier.Mode.OPTIMISTIC));
        
        verifier.setMode(NovaVerifier.Mode.FULL);
        assertEq(uint(verifier.mode()), uint(NovaVerifier.Mode.FULL));
    }
    
    function test_SetMode_RevertIfNotOwner() public {
        vm.prank(address(0x123));
        vm.expectRevert("Not owner");
        verifier.setMode(NovaVerifier.Mode.OPTIMISTIC);
    }
    
    //==========================================================================
    // OPTIMISTIC MODE TESTS
    //==========================================================================
    
    function test_OptimisticMode_RequiresSubmission() public {
        verifier.setMode(NovaVerifier.Mode.OPTIMISTIC);
        
        bytes memory proof = new bytes(100);
        
        // Should fail because not submitted yet
        bool valid = verifier.verify(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            threshold,
            proof
        );
        
        assertFalse(valid);
    }
    
    function test_OptimisticMode_SubmitAndWait() public {
        verifier.setMode(NovaVerifier.Mode.OPTIMISTIC);
        
        bytes memory proof = new bytes(100);
        
        // Submit for optimistic verification
        verifier.submitForOptimisticVerification(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            proof
        );
        
        // Still not valid (challenge period not passed)
        bool validBefore = verifier.verify(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            threshold,
            proof
        );
        assertFalse(validBefore);
        
        // Fast forward past challenge period
        vm.warp(block.timestamp + 2 hours);
        
        // Now should be valid
        bool validAfter = verifier.verify(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            threshold,
            proof
        );
        assertTrue(validAfter);
    }
    
    function test_OptimisticMode_ChallengeProof() public {
        verifier.setMode(NovaVerifier.Mode.OPTIMISTIC);
        
        bytes memory proof = new bytes(100);
        bytes memory fraudProof = new bytes(50);
        fraudProof[0] = 0x01;
        
        // Submit for optimistic verification
        verifier.submitForOptimisticVerification(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            proof
        );
        
        // Challenge the proof
        verifier.challengeProof(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            proof,
            fraudProof
        );
        
        // Fast forward past challenge period
        vm.warp(block.timestamp + 2 hours);
        
        // Should be invalid because it was challenged
        bool valid = verifier.verify(
            problemCommitment,
            spinCommitment,
            claimedEnergy,
            threshold,
            proof
        );
        assertFalse(valid);
    }
    
    //==========================================================================
    // ADMIN TESTS
    //==========================================================================
    
    function test_SetChallengePeriod() public {
        verifier.setChallengePeriod(2 hours);
        assertEq(verifier.challengePeriod(), 2 hours);
    }
    
    function test_SetChallengePeriod_RevertIfTooShort() public {
        vm.expectRevert("Period too short");
        verifier.setChallengePeriod(5 minutes);
    }
    
    function test_TransferOwnership() public {
        address newOwner = address(0x999);
        verifier.transferOwnership(newOwner);
        assertEq(verifier.owner(), newOwner);
    }
}
