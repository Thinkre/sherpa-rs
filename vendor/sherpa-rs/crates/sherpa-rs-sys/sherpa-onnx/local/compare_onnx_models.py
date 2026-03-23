#!/usr/bin/env python3
# -*- encoding: utf-8 -*-
"""
Compare two ONNX model structures (inputs, outputs, metadata, graph structure)
without comparing weights.

Usage:
    python3 local/compare_onnx_models.py \
        --model1 models/sherpa-onnx-paraformer-zh-int8-2025-10-07/model.int8.onnx \
        --model2 ~/Desktop/models/beike/ke_paraformer_0902_onnx/model.onnx
"""

import argparse
import json
from pathlib import Path
from typing import Dict, List, Set, Tuple

try:
    import onnx
    from onnx import helper
    USE_ONNX = True
except ImportError:
    try:
        import onnxruntime as ort
        USE_ONNX = False
    except ImportError:
        print("Error: Neither 'onnx' nor 'onnxruntime' package found.")
        print("Please install one of them:")
        print("  pip install onnx")
        print("  or")
        print("  pip install onnxruntime")
        exit(1)


def get_model_info(model_path: str) -> Dict:
    """Extract model structure information."""
    if USE_ONNX:
        return get_model_info_onnx(model_path)
    else:
        return get_model_info_ort(model_path)


def get_model_info_onnx(model_path: str) -> Dict:
    """Extract model structure information using onnx library."""
    model = onnx.load(model_path)
    info = {
        "ir_version": model.ir_version,
        "opset_import": [],
        "producer_name": model.producer_name,
        "producer_version": model.producer_version,
        "domain": model.domain,
        "model_version": model.model_version,
        "doc_string": model.doc_string,
        "metadata_props": {},
        "inputs": [],
        "outputs": [],
        "node_count": len(model.graph.node),
        "node_types": {},
        "initializers": [],
    }

    # Opset imports
    for opset in model.opset_import:
        info["opset_import"].append({
            "domain": opset.domain,
            "version": opset.version,
        })

    # Metadata properties
    for prop in model.metadata_props:
        info["metadata_props"][prop.key] = prop.value

    # Inputs
    for inp in model.graph.input:
        shape = []
        if inp.type.tensor_type.shape.dim:
            for dim in inp.type.tensor_type.shape.dim:
                if dim.dim_value:
                    shape.append(dim.dim_value)
                elif dim.dim_param:
                    shape.append(dim.dim_param)
                else:
                    shape.append("?")
        info["inputs"].append({
            "name": inp.name,
            "type": helper.tensor_dtype_to_np_dtype(inp.type.tensor_type.elem_type).__name__,
            "shape": shape,
        })

    # Outputs
    for out in model.graph.output:
        shape = []
        if out.type.tensor_type.shape.dim:
            for dim in out.type.tensor_type.shape.dim:
                if dim.dim_value:
                    shape.append(dim.dim_value)
                elif dim.dim_param:
                    shape.append(dim.dim_param)
                else:
                    shape.append("?")
        info["outputs"].append({
            "name": out.name,
            "type": helper.tensor_dtype_to_np_dtype(out.type.tensor_type.elem_type).__name__,
            "shape": shape,
        })

    # Node types
    for node in model.graph.node:
        op_type = node.op_type
        info["node_types"][op_type] = info["node_types"].get(op_type, 0) + 1

    # Initializers (weights) - only names and shapes, not values
    for init in model.graph.initializer:
        info["initializers"].append({
            "name": init.name,
            "dtype": helper.tensor_dtype_to_np_dtype(init.data_type).__name__,
            "shape": list(init.dims),
        })

    return info


def get_model_info_ort(model_path: str) -> Dict:
    """Extract model structure information using onnxruntime."""
    import onnxruntime as ort
    
    sess = ort.InferenceSession(model_path, providers=['CPUExecutionProvider'])
    model_meta = sess.get_modelmeta()
    
    info = {
        "ir_version": "N/A (using onnxruntime)",
        "opset_import": [],
        "producer_name": model_meta.producer_name,
        "producer_version": model_meta.producer_version,
        "domain": model_meta.domain,
        "model_version": model_meta.version,
        "doc_string": model_meta.description,
        "metadata_props": {},
        "inputs": [],
        "outputs": [],
        "node_count": "N/A",
        "node_types": {},
        "initializers": [],
    }
    
    # Get custom metadata
    for key in model_meta.custom_metadata_map:
        info["metadata_props"][key] = model_meta.custom_metadata_map[key]
    
    # Inputs
    for inp in sess.get_inputs():
        info["inputs"].append({
            "name": inp.name,
            "type": str(inp.type),
            "shape": list(inp.shape),
        })
    
    # Outputs
    for out in sess.get_outputs():
        info["outputs"].append({
            "name": out.name,
            "type": str(out.type),
            "shape": list(out.shape),
        })
    
    # Note: onnxruntime doesn't provide detailed graph structure
    # We can't get node types or initializer info easily
    
    return info


def print_section(title: str):
    """Print a section header."""
    print("\n" + "=" * 80)
    print(f"  {title}")
    print("=" * 80)


def compare_models(info1: Dict, info2: Dict, name1: str, name2: str):
    """Compare two model structures."""
    print_section("Model Comparison")

    # Basic info
    print("\n[Basic Information]")
    print(f"{'Property':<30} {name1:<40} {name2:<40}")
    print("-" * 110)
    print(f"{'IR Version':<30} {info1['ir_version']:<40} {info2['ir_version']:<40}")
    print(f"{'Producer':<30} {info1['producer_name']:<40} {info2['producer_name']:<40}")
    print(f"{'Producer Version':<30} {info1['producer_version']:<40} {info2['producer_version']:<40}")
    print(f"{'Domain':<30} {info1['domain']:<40} {info2['domain']:<40}")
    print(f"{'Model Version':<30} {info1['model_version']:<40} {info2['model_version']:<40}")

    # Opset imports
    print_section("Opset Imports")
    print(f"\n{name1}:")
    for opset in info1["opset_import"]:
        print(f"  - {opset['domain']}: version {opset['version']}")
    print(f"\n{name2}:")
    for opset in info2["opset_import"]:
        print(f"  - {opset['domain']}: version {opset['version']}")

    # Metadata properties
    print_section("Metadata Properties")
    all_keys = set(info1["metadata_props"].keys()) | set(info2["metadata_props"].keys())
    
    print(f"\n{'Key':<40} {name1:<40} {name2:<40}")
    print("-" * 120)
    for key in sorted(all_keys):
        val1 = info1["metadata_props"].get(key, "<not present>")
        val2 = info2["metadata_props"].get(key, "<not present>")
        marker = "⚠️" if val1 != val2 else "✓"
        print(f"{marker} {key:<38} {str(val1)[:38]:<40} {str(val2)[:38]:<40}")

    # Inputs
    print_section("Inputs")
    print(f"\n{name1}:")
    for inp in info1["inputs"]:
        print(f"  - {inp['name']}: {inp['type']} {inp['shape']}")
    print(f"\n{name2}:")
    for inp in info2["inputs"]:
        print(f"  - {inp['name']}: {inp['type']} {inp['shape']}")

    # Outputs
    print_section("Outputs")
    print(f"\n{name1}:")
    for out in info1["outputs"]:
        print(f"  - {out['name']}: {out['type']} {out['shape']}")
    print(f"\n{name2}:")
    for out in info2["outputs"]:
        print(f"  - {out['name']}: {out['type']} {out['shape']}")

    # Node types
    print_section("Node Types (Operator Count)")
    all_node_types = set(info1["node_types"].keys()) | set(info2["node_types"].keys())
    print(f"\n{'Operator':<30} {name1:<20} {name2:<20} {'Match':<10}")
    print("-" * 80)
    for op_type in sorted(all_node_types):
        count1 = info1["node_types"].get(op_type, 0)
        count2 = info2["node_types"].get(op_type, 0)
        match = "✓" if count1 == count2 else "⚠️"
        print(f"{op_type:<30} {count1:<20} {count2:<20} {match:<10}")

    print(f"\nTotal nodes: {info1['node_count']} vs {info2['node_count']}")

    # Initializers (weights) - only structure
    print_section("Initializers (Weights) - Structure Only")
    print(f"\n{name1}: {len(info1['initializers'])} initializers")
    print(f"{name2}: {len(info2['initializers'])} initializers")
    
    # Compare initializer names
    init_names1 = {init["name"] for init in info1["initializers"]}
    init_names2 = {init["name"] for init in info2["initializers"]}
    
    only_in_1 = init_names1 - init_names2
    only_in_2 = init_names2 - init_names1
    common = init_names1 & init_names2
    
    print(f"\nCommon initializers: {len(common)}")
    print(f"Only in {name1}: {len(only_in_1)}")
    print(f"Only in {name2}: {len(only_in_2)}")
    
    if only_in_1:
        print(f"\n⚠️  Initializers only in {name1}:")
        for name in sorted(list(only_in_1))[:10]:  # Show first 10
            print(f"    - {name}")
        if len(only_in_1) > 10:
            print(f"    ... and {len(only_in_1) - 10} more")
    
    if only_in_2:
        print(f"\n⚠️  Initializers only in {name2}:")
        for name in sorted(list(only_in_2))[:10]:  # Show first 10
            print(f"    - {name}")
        if len(only_in_2) > 10:
            print(f"    ... and {len(only_in_2) - 10} more")


def main():
    parser = argparse.ArgumentParser(
        description="Compare two ONNX model structures"
    )
    parser.add_argument(
        "--model1",
        type=str,
        required=True,
        help="Path to first ONNX model",
    )
    parser.add_argument(
        "--model2",
        type=str,
        required=True,
        help="Path to second ONNX model",
    )
    parser.add_argument(
        "--json",
        type=str,
        help="Optional: Save comparison to JSON file",
    )

    args = parser.parse_args()

    model1_path = Path(args.model1).expanduser().resolve()
    model2_path = Path(args.model2).expanduser().resolve()

    if not model1_path.exists():
        print(f"Error: Model 1 not found: {model1_path}")
        return 1

    if not model2_path.exists():
        print(f"Error: Model 2 not found: {model2_path}")
        return 1

    print(f"Loading model 1: {model1_path}")
    info1 = get_model_info(str(model1_path))

    print(f"Loading model 2: {model2_path}")
    info2 = get_model_info(str(model2_path))

    name1 = model1_path.name
    name2 = model2_path.name

    compare_models(info1, info2, name1, name2)

    if args.json:
        output = {
            "model1": {"path": str(model1_path), "info": info1},
            "model2": {"path": str(model2_path), "info": info2},
        }
        with open(args.json, "w", encoding="utf-8") as f:
            json.dump(output, f, indent=2, default=str)
        print(f"\n✓ Comparison saved to: {args.json}")

    return 0


if __name__ == "__main__":
    exit(main())
