// Debug utilities for sherpa-onnx library verification

use std::ffi::CStr;
use std::io::Write;

/// Verify sherpa-onnx library is loaded and accessible
pub fn verify_sherpa_library() {
    eprintln!("[DEBUG] [LIB_VERIFY] Verifying sherpa-onnx library...");
    let _ = std::io::stderr().flush();
    
    // Try to get function pointer (this will fail if library is not loaded)
    unsafe {
        use sherpa_rs_sys::*;
        
        // Check if function pointers are valid
        let create_fn = SherpaOnnxCreateOfflineRecognizer as *const ();
        eprintln!("[DEBUG] [LIB_VERIFY] SherpaOnnxCreateOfflineRecognizer function address: {:p}", create_fn);
        
        // Try to create a minimal test config to see if library is functional
        // We'll just check if we can access the function, not actually call it
        eprintln!("[DEBUG] [LIB_VERIFY] Library function accessible: {}", !create_fn.is_null());
        
        let _ = std::io::stderr().flush();
    }
    
    // Check environment variables
    eprintln!("[DEBUG] [LIB_VERIFY] Environment variables:");
    eprintln!("[DEBUG] [LIB_VERIFY]   SHERPA_LIB_PATH: {:?}", std::env::var("SHERPA_LIB_PATH").ok());
    eprintln!("[DEBUG] [LIB_VERIFY]   DYLD_LIBRARY_PATH: {:?}", std::env::var("DYLD_LIBRARY_PATH").ok());
    eprintln!("[DEBUG] [LIB_VERIFY]   LD_LIBRARY_PATH: {:?}", std::env::var("LD_LIBRARY_PATH").ok());
    
    let _ = std::io::stderr().flush();
}
