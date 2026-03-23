#!/usr/bin/env python3
"""
简化版 SeACo Paraformer 导出脚本
使用 PyTorch 直接导出到 ONNX
"""

import sys
import json
import torch
from pathlib import Path


def export_model(model_dir, output_dir):
    """导出模型到 ONNX"""
    model_dir = Path(model_dir)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    model_pt_path = model_dir / "model.pt"
    config_json_path = model_dir / "configuration.json"
    
    if not model_pt_path.exists():
        print(f"错误: 模型文件不存在: {model_pt_path}")
        return False
    
    print(f"正在加载模型: {model_pt_path}")
    
    # 加载 PyTorch 模型
    try:
        checkpoint = torch.load(model_pt_path, map_location='cpu')
        print("✓ 模型文件加载成功")
    except Exception as e:
        print(f"错误: 无法加载模型文件: {e}")
        return False
    
    # 提取模型对象
    if isinstance(checkpoint, dict):
        # 检查常见的模型键
        model_obj = None
        for key in ['model', 'state_dict', 'model_state_dict', 'net']:
            if key in checkpoint:
                if isinstance(checkpoint[key], torch.nn.Module):
                    model_obj = checkpoint[key]
                    break
                elif isinstance(checkpoint[key], dict):
                    # 需要模型结构来加载 state_dict
                    print(f"找到 state_dict，但需要模型结构来加载")
                    print("提示: 请使用 FunASR 的导出工具:")
                    print(f"  pip install -U modelscope funasr")
                    print(f"  python -m funasr.export.export_model \\")
                    print(f"    --model-name {model_dir} \\")
                    print(f"    --export-dir {output_dir} \\")
                    print(f"    --type onnx")
                    return False
        
        if model_obj is None:
            print("错误: 无法从 checkpoint 中提取模型对象")
            print("提示: 请使用 FunASR 的导出工具:")
            print(f"  pip install -U modelscope funasr")
            print(f"  python -m funasr.export.export_model \\")
            print(f"    --model-name {model_dir} \\")
            print(f"    --export-dir {output_dir} \\")
    else:
        model_obj = checkpoint
    
    if model_obj is None:
        print("错误: 无法获取模型对象")
        return False
    
    # 设置为评估模式
    if hasattr(model_obj, 'eval'):
        model_obj.eval()
    
    # 准备 dummy 输入
    # Paraformer 的输入通常是音频特征: (batch, seq_len, feature_dim)
    # 或者可能是原始音频: (batch, audio_samples)
    print("准备示例输入...")
    
    # 尝试不同的输入格式
    dummy_inputs = [
        torch.randn(1, 100, 80),  # (batch, seq_len, feature_dim) - 特征输入
        torch.randn(1, 16000),    # (batch, audio_samples) - 原始音频 1秒@16kHz
    ]
    
    onnx_path = output_dir / "model.onnx"
    exported = False
    
    for i, dummy_input in enumerate(dummy_inputs):
        try:
            print(f"尝试输入格式 {i+1}: {dummy_input.shape}...")
            
            # 尝试前向传播，确保输入格式正确
            with torch.no_grad():
                _ = model_obj(dummy_input)
            
            # 导出 ONNX
            torch.onnx.export(
                model_obj,
                dummy_input,
                str(onnx_path),
                input_names=['speech'],
                output_names=['text'],
                dynamic_axes={
                    'speech': {0: 'batch', 1: 'sequence'},
                },
                opset_version=13,
                do_constant_folding=True,
                verbose=False,
            )
            
            print(f"✓ ONNX 模型已导出: {onnx_path}")
            exported = True
            break
            
        except Exception as e:
            print(f"  失败: {e}")
            continue
    
    if not exported:
        print("\n错误: 无法导出模型")
        print("\n建议使用 FunASR 官方导出工具:")
        print(f"  pip install -U modelscope funasr")
        print(f"  python -m funasr.export.export_model \\")
        print(f"    --model-name {model_dir} \\")
        print(f"    --export-dir {output_dir} \\")
        print(f"    --type onnx \\")
        print(f"    --quantize False")
        return False
    
    # 提取 tokens.txt
    print("\n提取 tokens.txt...")
    tokens_txt_path = output_dir / "tokens.txt"
    
    # 尝试从 ModelScope 下载 tokens.json
    if config_json_path.exists():
        try:
            with open(config_json_path, 'r', encoding='utf-8') as f:
                config_json = json.load(f)
            
            model_name = config_json.get("model_name_in_hub", {}).get("ms", "")
            if model_name:
                print(f"从 ModelScope 下载 tokens: {model_name}")
                try:
                    from modelscope import snapshot_download
                    cache_dir = snapshot_download(model_name, cache_dir=str(model_dir / ".cache"))
                    tokens_json_path = Path(cache_dir) / "tokens.json"
                    
                    if tokens_json_path.exists():
                        with open(tokens_json_path, 'r', encoding='utf-8') as f:
                            tokens_data = json.load(f)
                        
                        tokens = None
                        if isinstance(tokens_data, dict):
                            tokens = tokens_data.get("token_list", tokens_data.get("tokens", []))
                        elif isinstance(tokens_data, list):
                            tokens = tokens_data
                        
                        if tokens and isinstance(tokens, list):
                            with open(tokens_txt_path, 'w', encoding='utf-8') as f:
                                for token in tokens:
                                    f.write(f"{token}\n")
                            print(f"✓ 已提取 {len(tokens)} 个 tokens")
                            return True
                except ImportError:
                    print("需要安装 modelscope: pip install modelscope")
                except Exception as e:
                    print(f"下载 tokens 失败: {e}")
        except Exception as e:
            print(f"读取配置失败: {e}")
    
    # 如果无法提取，创建占位符
    if not tokens_txt_path.exists():
        print("警告: 无法提取 tokens.txt")
        print("请手动从 ModelScope 下载 tokens.json 并转换为 tokens.txt")
        print(f"模型名称: {model_name if 'model_name' in locals() else '未知'}")
    
    # 复制其他文件
    print("\n复制其他文件...")
    import shutil
    for file_name in ["am.mvn", "config.yaml"]:
        src_file = model_dir / file_name
        if src_file.exists():
            shutil.copy(src_file, output_dir / file_name)
            print(f"✓ 复制 {file_name}")
    
    print(f"\n✓ 导出完成！")
    print(f"输出目录: {output_dir}")
    return True


def main():
    if len(sys.argv) < 2:
        print("用法: python export_seaco_simple.py <model_dir> [output_dir]")
        print("示例: python export_seaco_simple.py ~/Desktop/models/beike/seaco_paraformer.20250904.for_general")
        sys.exit(1)
    
    model_dir = Path(sys.argv[1]).expanduser().resolve()
    output_dir = Path(sys.argv[2]).expanduser().resolve() if len(sys.argv) > 2 else model_dir / "onnx_export"
    
    if not model_dir.exists():
        print(f"错误: 模型目录不存在: {model_dir}")
        sys.exit(1)
    
    success = export_model(model_dir, output_dir)
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
