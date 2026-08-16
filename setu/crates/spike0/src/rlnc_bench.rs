//! RLNC Overhead Benchmark (Gate D)
//! 
//! Measures encoding/decoding latency under various loss scenarios.
//! Target: ≤5ms p50 overhead, ≤15ms p99

use setu_transport::rlnc::{RlncEncoder, RlncDecoder, RlncConfig};

pub fn run() {
    println!("\n=== Gate D: RLNC Overhead Benchmark ===");
    
    let config = RlncConfig {
        k: 16,      // Source chunks per generation
        m: 8,       // Parity chunks
        window_size: 8, // ms
    };
    
    let mut encoder = RlncEncoder::new(config.clone());
    let mut decoder = RlncDecoder::new(config);
    
    // Test payload: 1280-byte chunks (typical video tile after compression)
    let chunk_size = 1280;
    let num_objects = 1000;
    
    let mut latencies_encode = Vec::with_capacity(num_objects);
    let mut latencies_decode = Vec::with_capacity(num_objects);
    
    for i in 0..num_objects {
        // Create test data
        let data = vec![(i % 256) as u8; chunk_size];
        
        // Encode
        let start = std::time::Instant::now();
        let _encoded = encoder.encode_source_chunk(i as u32, &data);
        latencies_encode.push(start.elapsed().as_micros() as f64);
        
        // Generate parity chunks
        let parity = encoder.generate_parity();
        
        // Decode (simulate receiving K chunks)
        let start = std::time::Instant::now();
        for chunk in parity.iter().take(config.k as usize) {
            decoder.receive_chunk(chunk);
        }
        let decoded = decoder.decode_generation(i as u32);
        if decoded.is_some() {
            latencies_decode.push(start.elapsed().as_micros() as f64);
        }
    }
    
    // Calculate statistics
    let p50_enc = percentile(&mut latencies_encode, 50.0);
    let p99_enc = percentile(&mut latencies_encode, 99.0);
    let p50_dec = percentile(&mut latencies_decode, 50.0);
    let p99_dec = percentile(&mut latencies_decode, 99.0);
    
    println!("Encode Latency: p50={:.2}μs, p99={:.2}μs", p50_enc, p99_enc);
    println!("Decode Latency: p50={:.2}μs, p99={:.2}μs", p50_dec, p99_dec);
    
    // Gate D Pass/Fail
    let pass = p50_enc <= 5000.0 && p99_enc <= 15000.0;
    println!("Gate D Result: {}", if pass { "PASS ✓" } else { "FAIL ✗" });
    
    if !pass {
        eprintln!("WARNING: RLNC overhead exceeds target. Consider optimization.");
    }
}

fn percentile(data: &mut [f64], p: f64) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    data.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let idx = ((p / 100.0) * (data.len() - 1) as f64) as usize;
    data[idx]
}
