// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title ISP1Verifier
 * @notice Interface for SP1 on-chain verifier
 * @dev Deployed by Succinct on various chains
 */
interface ISP1Verifier {
    /**
     * @notice Verify a SP1 proof
     * @param programVKey The verification key hash for the program
     * @param publicValues The encoded public values
     * @param proof The compressed STARK proof
     */
    function verifyProof(
        bytes32 programVKey,
        bytes calldata publicValues,
        bytes calldata proof
    ) external view;
}
