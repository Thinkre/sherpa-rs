#!/usr/bin/env python3
"""
导出 SeACo Paraformer PyTorch 模型到 ONNX 格式
使用 FunASR 的导出工具
"""

import os
import sys
import json
import subprocess
from pathlib import Path

try:
    from funasr import AutoModel
    from modelscope import snapshot_download
except ImportError:
    print("错误: 请先安装依赖:")
    print("  pip install -U modelscope funasr")
    sys.exit(1)


def extract_tokens(model_dir, output_dir):
    """提取 tokens.txt 文件"""
    tokens_txt_path = output_dir / "tokens.txt"
    
    # 方法1: 尝试从 tokens.json 提取
    tokens_json_path = model_dir / "tokens.json"
    if tokens_json_path.exists():
        print(f"从 {tokens_json_path} 提取 tokens...")
        with open(tokens_json_path, 'r', encoding='utf-8') as f:
            tokens_data = json.load(f)
        
        # tokens.json 格式可能是 {"token_list": [...]} 或直接是列表
        if isinstance(tokens_data, dict):
            if "token_list" in tokens_data:
                tokens = tokens_data["token_list"]
            elif "tokens" in tokens_data:
                tokens = tokens_data["tokens"]
            else:
                # 尝试找到包含 token 列表的键
                tokens = None
                for key, value in tokens_data.items():
                    if isinstance(value, list) and len(value) > 0:
                        tokens = value
                        break
                if tokens is None:
                    print(f"警告: 无法在 {tokens_json_path} 中找到 token 列表")
                    return False
        elif isinstance(tokens_data, list):
            tokens = tokens_data
        else:
            print(f"警告: 不支持的 tokens.json 格式")
            return False
        
        # 写入 tokens.txt（每行一个 token）
        with open(tokens_txt_path, 'w', encoding='utf-8') as f:
            for token in tokens:
                f.write(f"{token}\n")
        print(f"✓ 已提取 {len(tokens)} 个 tokens 到 {tokens_txt_path}")
        return True
    
    # 方法2: 尝试从 ModelScope 下载 tokens.json
    print("尝试从 ModelScope 下载 tokens.json...")
    try:
        config_json_path = model_dir / "configuration.json"
        if config_json_path.exists():
            with open(config_json_path, 'r', encoding='utf-8') as f:
                config_json = json.load(f)
            
            model_name = config_json.get("model_name_in_hub", {}).get("ms", "")
            if model_name:
                print(f"从 ModelScope 下载模型文件: {model_name}")
                cache_dir = snapshot_download(model_name, cache_dir=str(model_dir / ".cache"))
                tokens_json_path = Path(cache_dir) / "tokens.json"
                
                if tokens_json_path.exists():
                    # 复制到输出目录
                    import shutil
                    shutil.copy(tokens_json_path, output_dir / "tokens.json")
                    return extract_tokens(output_dir, output_dir)
    except Exception as e:
        print(f"从 ModelScope 下载失败: {e}")
    
    # 方法3: 尝试从模型加载并提取
    print("尝试从模型对象提取 tokens...")
    try:
        model = AutoModel(model=str(model_dir))
        if hasattr(model, 'tokenizer') and model.tokenizer is not None:
            tokenizer = model.tokenizer
            if hasattr(tokenizer, 'token_list'):
                tokens = tokenizer.token_list
                with open(tokens_txt_path, 'w', encoding='utf-8') as f:
                    for token in tokens:
                        f.write(f"{token}\n")
                print(f"✓ 从模型提取了 {len(tokens)} 个 tokens")
                return True
    except Exception as e:
        print(f"从模型提取 tokens 失败: {e}")
    
    print("警告: 无法提取 tokens.txt，请手动创建或从 ModelScope 下载")
    return False


def export_model_to_onnx(model_dir, output_dir):
    """导出模型到 ONNX"""
    model_dir = Path(model_dir)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    config_path = model_dir / "config.yaml"
    model_pt_path = model_dir / "model.pt"
    config_json_path = model_dir / "configuration.json"
    
    if not model_pt_path.exists():
        raise FileNotFoundError(f"模型文件不存在: {model_pt_path}")
    
    if not config_path.exists():
        raise FileNotFoundError(f"配置文件不存在: {config_path}")
    
    print(f"模型目录: {model_dir}")
    print(f"输出目录: {output_dir}")
    print()
    
    # 方法1: 使用 FunASR 的导出命令
    print("=" * 60)
    print("方法1: 使用 FunASR 导出命令")
    print("=" * 60)
    
    try:
        # 检查是否有 ModelScope 模型名称
        model_name = None
        if config_json_path.exists():
            with open(config_json_path, 'r', encoding='utf-8') as f:
                config_json = json.load(f)
            model_name = config_json.get("model_name_in_hub", {}).get("ms", "")
        
        # 使用 FunASR 的导出模块
        cmd = [
            sys.executable, "-m", "funasr.export.export_model",
            "--model-name", str(model_dir),
            "--export-dir", str(output_dir),
            "--type", "onnx",
            "--quantize", "False",
        ]
        
        print(f"执行命令: {' '.join(cmd)}")
        result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(model_dir))
        
        if result.returncode == 0:
            print("✓ FunASR 导出成功")
            print(result.stdout)
        else:
            print(f"FunASR 导出失败: {result.stderr}")
            print("尝试方法2...")
            raise Exception("FunASR export failed")
            
    except Exception as e:
        print(f"方法1 失败: {e}")
        print()
        print("=" * 60)
        print("方法2: 使用 AutoModel 加载后导出")
        print("=" * 60)
        
        # 方法2: 使用 AutoModel 加载模型
        try:
            print("加载模型...")
            model = AutoModel(model=str(model_dir))
            print("✓ 模型加载成功")
            
            # 尝试使用模型的导出方法
            if hasattr(model, 'export'):
                print("使用模型的 export 方法...")
                model.export(str(output_dir), type='onnx')
                print("✓ 导出成功")
            else:
                print("模型没有 export 方法，请使用 FunASR 命令行工具")
                return False
                
        except Exception as e2:
            print(f"方法2 也失败: {e2}")
            print("\n请手动使用以下命令导出:")
            print(f"  python -m funasr.export.export_model \\")
            print(f"    --model-name {model_dir} \\")
            print(f"    --export-dir {output_dir} \\")
            print(f"    --type onnx \\")
            print(f"    --quantize False")
            return False
    
    # 检查导出的文件
    onnx_file = output_dir / "model.onnx"
    if not onnx_file.exists():
        # 尝试查找其他可能的文件名
        onnx_files = list(output_dir.glob("*.onnx"))
        if onnx_files:
            onnx_file = onnx_files[0]
            # 重命名为 model.onnx
            onnx_file.rename(output_dir / "model.onnx")
            print(f"✓ 重命名 ONNX 文件为 model.onnx")
        else:
            print("错误: 未找到导出的 ONNX 文件")
            return False
    
    print(f"✓ ONNX 模型文件: {onnx_file}")
    
    # 提取 tokens.txt
    print()
    print("=" * 60)
    print("提取 tokens.txt")
    print("=" * 60)
    
    if not extract_tokens(model_dir, output_dir):
        # 如果提取失败，尝试从导出的目录中查找
        tokens_files = list(output_dir.glob("tokens.*"))
        if tokens_files:
            tokens_file = tokens_files[0]
            if tokens_file.suffix == ".json":
                # 从 JSON 转换为 TXT
                with open(tokens_file, 'r', encoding='utf-8') as f:
                    tokens_data = json.load(f)
                tokens = tokens_data.get("token_list", tokens_data.get("tokens", []))
                if isinstance(tokens, list):
                    tokens_txt_path = output_dir / "tokens.txt"
                    with open(tokens_txt_path, 'w', encoding='utf-8') as f:
                        for token in tokens:
                            f.write(f"{token}\n")
                    print(f"✓ 从 {tokens_file.name} 提取了 tokens.txt")
                else:
                    print(f"警告: 无法从 {tokens_file} 提取 tokens")
            elif tokens_file.suffix == ".txt":
                print(f"✓ 找到 tokens.txt: {tokens_file}")
        else:
            print("警告: 无法找到或提取 tokens.txt")
            print("提示: 请手动创建 tokens.txt 文件")
    
    # 复制其他必要文件
    print()
    print("=" * 60)
    print("复制其他必要文件")
    print("=" * 60)
    
    files_to_copy = ["am.mvn", "config.yaml"]
    for file_name in files_to_copy:
        src_file = model_dir / file_name
        if src_file.exists():
            import shutil
            shutil.copy(src_file, output_dir / file_name)
            print(f"✓ 复制 {file_name}")
    
    print()
    print("=" * 60)
    print("✓ 导出完成！")
    print("=" * 60)
    print(f"输出目录: {output_dir}")
    print(f"  - model.onnx: {'✓' if (output_dir / 'model.onnx').exists() else '✗'}")
    print(f"  - tokens.txt: {'✓' if (output_dir / 'tokens.txt').exists() else '✗'}")
    print(f"  - am.mvn: {'✓' if (output_dir / 'am.mvn').exists() else '✗'}")
    
    return True


def main():
    if len(sys.argv) < 2:
        print("用法: python export_seaco_paraformer.py <model_dir> [output_dir]")
        print("示例: python export_seaco_paraformer.py ~/Desktop/models/beike/seaco_paraformer.20250904.for_general")
        sys.exit(1)
    
    model_dir = Path(sys.argv[1]).expanduser().resolve()
    output_dir = Path(sys.argv[2]).expanduser().resolve() if len(sys.argv) > 2 else model_dir / "onnx_export"
    
    if not model_dir.exists():
        print(f"错误: 模型目录不存在: {model_dir}")
        sys.exit(1)
    
    print(f"模型目录: {model_dir}")
    print(f"输出目录: {output_dir}")
    print()
    
    try:
        export_model_to_onnx(model_dir, output_dir)
    except Exception as e:
        print(f"错误: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
