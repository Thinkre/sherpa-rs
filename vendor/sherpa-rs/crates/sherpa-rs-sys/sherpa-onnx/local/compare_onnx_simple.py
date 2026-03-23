#!/usr/bin/env python3
# -*- encoding: utf-8 -*-
"""
Simple comparison of ONNX models using onnxruntime (if available) or basic file inspection.
"""

import argparse
import sys
from pathlib import Path

def compare_with_onnxruntime(model1_path: str, model2_path: str):
    """Compare using onnxruntime."""
    try:
        import onnxruntime as ort
    except ImportError:
        return False
    
    print("=" * 80)
    print("Comparing ONNX Models using onnxruntime")
    print("=" * 80)
    
    sess1 = ort.InferenceSession(model1_path, providers=['CPUExecutionProvider'])
    sess2 = ort.InferenceSession(model2_path, providers=['CPUExecutionProvider'])
    
    meta1 = sess1.get_modelmeta()
    meta2 = sess2.get_modelmeta()
    
    print("\n[Model Metadata]")
    print(f"{'Property':<30} {'Model 1':<40} {'Model 2':<40}")
    print("-" * 110)
    print(f"{'Producer':<30} {meta1.producer_name:<40} {meta2.producer_name:<40}")
    print(f"{'Producer Version':<30} {meta1.producer_version:<40} {meta2.producer_version:<40}")
    print(f"{'Domain':<30} {meta1.domain:<40} {meta2.domain:<40}")
    print(f"{'Version':<30} {str(meta1.version):<40} {str(meta2.version):<40}")
    print(f"{'Description':<30} {meta1.description[:38]:<40} {meta2.description[:38]:<40}")
    
    print("\n[Custom Metadata]")
    keys1 = set(meta1.custom_metadata_map.keys())
    keys2 = set(meta2.custom_metadata_map.keys())
    all_keys = keys1 | keys2
    
    print(f"{'Key':<40} {'Model 1':<40} {'Model 2':<40}")
    print("-" * 120)
    for key in sorted(all_keys):
        val1 = meta1.custom_metadata_map.get(key, "<not present>")
        val2 = meta2.custom_metadata_map.get(key, "<not present>")
        marker = "⚠️" if val1 != val2 else "✓"
        val1_str = str(val1)[:38] if len(str(val1)) > 38 else str(val1)
        val2_str = str(val2)[:38] if len(str(val2)) > 38 else str(val2)
        print(f"{marker} {key:<38} {val1_str:<40} {val2_str:<40}")
    
    print("\n[Inputs]")
    print(f"\nModel 1 ({len(sess1.get_inputs())} inputs):")
    for inp in sess1.get_inputs():
        print(f"  - {inp.name}: {inp.type} {list(inp.shape)}")
    
    print(f"\nModel 2 ({len(sess2.get_inputs())} inputs):")
    for inp in sess2.get_inputs():
        print(f"  - {inp.name}: {inp.type} {list(inp.shape)}")
    
    print("\n[Outputs]")
    print(f"\nModel 1 ({len(sess1.get_outputs())} outputs):")
    for out in sess1.get_outputs():
        print(f"  - {out.name}: {out.type} {list(out.shape)}")
    
    print(f"\nModel 2 ({len(sess2.get_outputs())} outputs):")
    for out in sess2.get_outputs():
        print(f"  - {out.name}: {out.type} {list(out.shape)}")
    
    return True


def main():
    parser = argparse.ArgumentParser(description="Compare ONNX model structures")
    parser.add_argument("--model1", required=True, help="Path to first model")
    parser.add_argument("--model2", required=True, help="Path to second model")
    
    args = parser.parse_args()
    
    model1_path = Path(args.model1).expanduser().resolve()
    model2_path = Path(args.model2).expanduser().resolve()
    
    if not model1_path.exists():
        print(f"Error: Model 1 not found: {model1_path}")
        return 1
    
    if not model2_path.exists():
        print(f"Error: Model 2 not found: {model2_path}")
        return 1
    
    if not compare_with_onnxruntime(str(model1_path), str(model2_path)):
        print("Error: onnxruntime not available. Cannot compare models.")
        print("Please install onnxruntime or use a different method.")
        return 1
    
    return 0


if __name__ == "__main__":
    exit(main())
