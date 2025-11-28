use crate::IsingCircuit;
use halo2_proofs::{
    plonk::{Circuit, keygen_vk, keygen_pk, create_proof, verify_proof, ProvingKey, VerifyingKey},
    poly::kzg::{
        commitment::ParamsKZG,
        multiopen::{ProverGWC, VerifierGWC},
    },
    transcript::{Blake2bRead, Blake2bWrite, Challenge255},
};
use halo2curves::bn256::{Bn256, G1Affine, Fr};
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;

/// Generate universal parameters for k rows.
/// NOTE: This assumes your circuit field is `Fr` (bn256::Fr).
pub fn setup_params(k: u32) -> ParamsKZG<Bn256> {
    ParamsKZG::<Bn256>::new(k)
}

/// Generate verifying key and proving key for the IsingCircuit.
pub fn keygen(
    params: &ParamsKZG<Bn256>,
    circuit: &IsingCircuit,  // or &impl Circuit<Fr>
) -> (VerifyingKey<G1Affine>, ProvingKey<G1Affine>) {
    let vk = keygen_vk(params, circuit).expect("vk generation should succeed");
    let pk = keygen_pk(params, vk.clone(), circuit).expect("pk generation should succeed");
    (vk, pk)
}

/// Create a proof for a single IsingCircuit instance.
pub fn create_ising_proof(
    params: &ParamsKZG<Bn256>,
    pk: &ProvingKey<G1Affine>,
    circuit: IsingCircuit,
    public_inputs: Vec<Vec<Fr>>,
) -> Vec<u8> {
    let mut rng = ChaCha20Rng::seed_from_u64(42);

    let mut transcript = Blake2bWrite::<_, G1Affine, Challenge255<_>>::init(vec![]);

    create_proof::<KZGCommitmentScheme<Bn256>, ProverGWC<_>, _, _, _, _>(
        params,
        pk,
        &[circuit],
        &[&public_inputs.iter().map(|v| &v[..]).collect::<Vec<_>>()],
        &mut rng,
        &mut transcript,
    )
    .expect("proof generation should succeed");

    transcript.finalize()
}

/// Verify a proof for an IsingCircuit instance.
pub fn verify_ising_proof(
    params: &ParamsKZG<Bn256>,
    vk: &VerifyingKey<G1Affine>,
    public_inputs: Vec<Vec<Fr>>,
    proof: &[u8],
) -> bool {
    let mut transcript = Blake2bRead::<_, G1Affine, Challenge255<_>>::init(proof);
    let strategy = SingleVerifier::new(params);

    verify_proof::<KZGCommitmentScheme<Bn256>, VerifierGWC<_>, _, _, _>(
        params,
        vk,
        strategy,
        &[&public_inputs.iter().map(|v| &v[..]).collect::<Vec<_>>()],
        &mut transcript,
    )
    .is_ok()
}
