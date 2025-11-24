// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

/**
 * @title SP1NovaVerifier
 * @notice Verifies STARK-wrapped Nova proofs on-chain
 * @dev Provides quantum-resistant verification of Ising optimization proofs
 */
contract SP1NovaVerifier {
    
    /// @notice SP1 verifier contract (deployed by Succinct)
    ISP1Verifier public immutable sp1Verifier;
    
    /// @notice Verification key hash for our Nova verifier program
    bytes32 public immutable programVKeyHash;
    
    /// @notice Owner for admin functions
    address public owner;
    
    /// @notice Verified output structure
    struct VerifiedOutput {
        bytes32 problemCommitment;
        bytes32 spinCommitment;
        int64 energy;
        int64 threshold;
        bool valid;
    }
    
    /// @notice Emitted when a proof is verified
    event ProofVerified(
        bytes32 indexed problemCommitment,
        bytes32 indexed spinCommitment,
        int64 energy,
        int64 threshold
    );
    
    constructor(address _sp1Verifier, bytes32 _programVKeyHash) {
        sp1Verifier = ISP1Verifier(_sp1Verifier);
        programVKeyHash = _programVKeyHash;
        owner = msg.sender;
    }
    
    /**
     * @notice Verify a STARK-wrapped Nova proof
     * @param proof The SP1 compressed proof
     * @param publicValues The encoded public values from the proof
     * @return output The verified output
     */
    function verifyProof(
        bytes calldata proof,
        bytes calldata publicValues
    ) external returns (VerifiedOutput memory output) {
        // Verify the STARK proof using SP1 verifier
        sp1Verifier.verifyProof(programVKeyHash, publicValues, proof);
        
        // Decode public values
        output = abi.decode(publicValues, (VerifiedOutput));
        
        // Ensure the proof is valid
        require(output.valid, "Proof marked invalid");
        require(output.energy <= output.threshold, "Energy exceeds threshold");
        
        emit ProofVerified(
            output.problemCommitment,
            output.spinCommitment,
            output.energy,
            output.threshold
        );
        
        return output;
    }
    
    /**
     * @notice Verify proof and check against expected job parameters
     * @param proof The SP1 compressed proof
     * @param publicValues The encoded public values
     * @param expectedProblem Expected problem commitment
     * @param expectedThreshold Expected threshold
     */
    function verifyProofForJob(
        bytes calldata proof,
        bytes calldata publicValues,
        bytes32 expectedProblem,
        int64 expectedThreshold
    ) external returns (VerifiedOutput memory output) {
        output = this.verifyProof(proof, publicValues);
        
        require(output.problemCommitment == expectedProblem, "Problem mismatch");
        require(output.threshold == expectedThreshold, "Threshold mismatch");
        
        return output;
    }
    
    /**
     * @notice Check if a proof would verify (view function)
     * @dev Uses try/catch to check without state changes
     */
    function wouldVerify(
        bytes calldata proof,
        bytes calldata publicValues
    ) external view returns (bool) {
        try sp1Verifier.verifyProof(programVKeyHash, publicValues, proof) {
            return true;
        } catch {
            return false;
        }
    }
}
