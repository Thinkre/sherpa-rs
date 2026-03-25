use sherpa_rs::paraformer::{ParaformerConfig, ParaformerRecognizer};
use std::path::Path;

fn main() {
    println!("=== Testing SeACo Paraformer ===");

    let model_dir = Path::new("/Users/thinkre/Library/Application Support/com.kevoiceinput.app/models/KeSeACoParaformer-2509");

    let model = model_dir.join("model.onnx").to_string_lossy().to_string();
    let tokens = model_dir.join("tokens.txt").to_string_lossy().to_string();
    let model_eb = model_dir.join("model_eb.onnx").to_string_lossy().to_string();

    println!("Model: {}", model);
    println!("Tokens: {}", tokens);
    println!("Model EB: {}", model_eb);

    println!("\nCreating ParaformerConfig...");

    let config = ParaformerConfig {
        model,
        tokens,
        model_eb: Some(model_eb),
        hotwords_file: None,
        hotwords_score: 0.0,
        provider: None,
        num_threads: Some(2),
        debug: true,
    };

    println!("Config created successfully!");
    println!("\nCreating ParaformerRecognizer...");

    match ParaformerRecognizer::new(config) {
        Ok(_recognizer) => {
            println!("✅ SeACo Paraformer recognizer created successfully!");
        }
        Err(e) => {
            eprintln!("❌ Failed to create recognizer: {:?}", e);
            std::process::exit(1);
        }
    }
}
