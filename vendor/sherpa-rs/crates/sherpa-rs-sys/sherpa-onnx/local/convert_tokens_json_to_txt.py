#!/usr/bin/env python3
# -*- encoding: utf-8 -*-
"""
Convert tokens.json (FunASR format) to tokens.txt (sherpa-onnx format)

Usage:
    python3 local/convert_tokens_json_to_txt.py \
        --input ~/Desktop/models/beike/ke_paraformer_0902_onnx/tokens.json

The output file will be saved as tokens.txt in the same directory as the input file.
"""

import argparse
import json
import os
from pathlib import Path


def convert_tokens_json_to_txt(input_file: str) -> str:
    """
    Convert tokens.json to tokens.txt format.

    Args:
        input_file: Path to tokens.json file

    Returns:
        Path to the output tokens.txt file
    """
    input_path = Path(input_file).expanduser().resolve()

    if not input_path.exists():
        raise FileNotFoundError(f"Input file does not exist: {input_file}")

    if not input_path.name.endswith(".json"):
        raise ValueError(f"Input file should be a .json file: {input_file}")

    # Read tokens.json
    print(f"Reading tokens from: {input_path}")
    with open(input_path, "r", encoding="utf-8") as f:
        tokens = json.load(f)

    if not isinstance(tokens, list):
        raise ValueError(
            f"tokens.json should contain a JSON array, but got {type(tokens)}"
        )

    # Generate output path (same directory, named tokens.txt)
    output_path = input_path.parent / "tokens.txt"

    # Write tokens.txt
    print(f"Writing tokens to: {output_path}")
    with open(output_path, "w", encoding="utf-8") as f:
        for idx, token in enumerate(tokens):
            # Format: token id
            # e.g., <blank> 0
            f.write(f"{token} {idx}\n")

    print(f"Successfully converted {len(tokens)} tokens")
    print(f"Output saved to: {output_path}")

    return str(output_path)


def main():
    parser = argparse.ArgumentParser(
        description="Convert tokens.json (FunASR format) to tokens.txt (sherpa-onnx format)"
    )
    parser.add_argument(
        "--input",
        type=str,
        required=True,
        help="Path to tokens.json file",
    )

    args = parser.parse_args()

    try:
        output_path = convert_tokens_json_to_txt(args.input)
        print(f"\n✓ Conversion completed successfully!")
        print(f"  Output: {output_path}")
    except Exception as e:
        print(f"\n✗ Error: {e}")
        return 1

    return 0


if __name__ == "__main__":
    exit(main())
