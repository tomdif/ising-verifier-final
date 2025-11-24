use neptune::batch_hasher::BatcherType;
use neptune::batch_hasher::Batcher;
use pasta_curves::Fp;
use generic_array::typenum::U2;

fn main() {
    println!("Checking for GPU...");
    
    // Try to detect GPU
    match BatcherType::pick_gpu_batch_size() {
        Some(size) => println!("GPU detected! Batch size: {}", size),
        None => println!("No GPU detected"),
    }
}
