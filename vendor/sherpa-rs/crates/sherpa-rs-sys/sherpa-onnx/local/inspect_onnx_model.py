#!/usr/bin/env python3
# -*- encoding: utf-8 -*-
"""
Inspect ONNX model structure using onnxruntime (if available) or provide instructions.
"""

import argparse
import sys
from pathlib import Path


def inspect_model(model_path: str):
    """Inspect a single ONNX model."""
    try:
        import onnxruntime as ort
    except ImportError:
        print("Error: onnxruntime not available.")
        print("\nTo inspect models, please install onnxruntime:")
        print("  pip install onnxruntime")
        print("\nOr use the C++ tool with debug mode:")
        print(f"  ./build/bin/sherpa-onnx-offline --paraformer={model_path} --debug=1 /dev/null")
        return None
    
    sess_opts = ort.SessionOptions()
    sess_opts.log_severity_level = 3  # Suppress warnings
    sess = ort.InferenceSession(model_path, sess_opts, providers=['CPUExecutionProvider'])
    meta = sess.get_modelmeta()
    
    info = {
        "producer": meta.producer_name,
        "producer_version": meta.producer_version,
        "domain": meta.domain,
        "version": meta.version,
        "description": meta.description,
        "metadata": dict(meta.custom_metadata_map),
        "inputs": [{"name": i.name, "type": str(i.type), "shape": list(i.shape)} for i in sess.get_inputs()],
        "outputs": [{"name": o.name, "type": str(o.type), "shape": list(o.shape)} for o in sess.get_outputs()],
    }
    
    return info


def print_model_info(info: dict, model_name: str):
    """Print model information."""
    print("\n" + "=" * 80)
    print(f"  {model_name}")
    print("=" * 80)
    
    print(f"\n[Basic Info]")
    print(f"  Producer: {info['producer']} {info['producer_version']}")
    print(f"  Domain: {info['domain']}")
    print(f"  Version: {info['version']}")
    if info['description']:
        print(f"  Description: {info['description']}")
    
    print(f"\n[Metadata Properties] ({len(info['metadata'])} items)")
    for key, value in sorted(info['metadata'].items()):
        print(f"  {key}: {value}")
    
    print(f"\n[Inputs] ({len(info['inputs'])} inputs)")
    for inp in info['inputs']:
        print(f"  - {inp['name']}: {inp['type']} {inp['shape']}")
    
    print(f"\n[Outputs] ({len(info['outputs'])} outputs)")
    for out in info['outputs']:
        print(f"  - {out['name']}: {out['type']} {out['shape']}")


def compare_models(info1: dict, info2: dict, name1: str, name2: str):
    """Compare two models."""
    print("\n" + "=" * 80)
    print("  COMPARISON")
    print("=" * 80)
    
    # Metadata comparison
    print("\n[Metadata Properties Comparison]")
    all_keys = set(info1['metadata'].keys()) | set(info2['metadata'].keys())
    print(f"\n{'Key':<40} {name1[:35]:<40} {name2[:35]:<40}")
    print("-" * 120)
    for key in sorted(all_keys):
        val1 = info1['metadata'].get(key, "<not present>")
        val2 = info2['metadata'].get(key, "<not present>")
        marker = "⚠️" if val1 != val2 else "✓"
        val1_str = str(val1)[:38] if len(str(val1)) > 38 else str(val1)
        val2_str = str(val2)[:38] if len(str(val2)) > 38 else str(val2)
        print(f"{marker} {key:<38} {val1_str:<40} {val2_str:<40}")
    
    # Inputs comparison
    print("\n[Inputs Comparison]")
    print(f"\n{name1}:")
    for inp in info1['inputs']:
        print(f"  - {inp['name']}: {inp['type']} {inp['shape']}")
    print(f"\n{name2}:")
    for inp in info2['inputs']:
        print(f"  - {inp['name']}: {inp['type']} {inp['shape']}")
    
    # Outputs comparison
    print("\n[Outputs Comparison]")
    print(f"\n{name1}:")
    for out in info1['outputs']:
        print(f"  - {out['name']}: {out['type']} {out['shape']}")
    print(f"\n{name2}:")
    for out in info2['outputs']:
        print(f"  - {out['name']}: {out['type']} {out['shape']}")


def main():
    parser = argparse.ArgumentParser(description="Inspect ONNX model structure")
    parser.add_argument("--model1", help="Path to first model")
    parser.add_argument("--model2", help="Path to second model (for comparison)")
    parser.add_argument("--model", help="Path to single model to inspect")
    
    args = parser.parse_args()
    
    if args.model:
        model_path = Path(args.model).expanduser().resolve()
        if not model_path.exists():
            print(f"Error: Model not found: {model_path}")
            return 1
        info = inspect_model(str(model_path))
        if info:
            print_model_info(info, model_path.name)
        return 0
    
    if args.model1 and args.model2:
        model1_path = Path(args.model1).expanduser().resolve()
        model2_path = Path(args.model2).expanduser().resolve()
        
        if not model1_path.exists():
            print(f"Error: Model 1 not found: {model1_path}")
            return 1
        if not model2_path.exists():
            print(f"Error: Model 2 not found: {model2_path}")
            return 1
        
        info1 = inspect_model(str(model1_path))
        info2 = inspect_model(str(model2_path))
        
        if info1 and info2:
            print_model_info(info1, model1_path.name)
            print_model_info(info2, model2_path.name)
            compare_models(info1, info2, model1_path.name, model2_path.name)
        return 0
    
    parser.print_help()
    return 1


if __name__ == "__main__":
    exit(main())
