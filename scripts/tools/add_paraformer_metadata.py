#!/usr/bin/env python3
"""
为 Paraformer ONNX 模型添加元数据（当元数据缺失时）
从 tokens.txt、am.mvn 和 config.yaml 读取信息并添加到 model.onnx 的元数据中
"""

import sys
import json
import yaml
import numpy as np
from pathlib import Path

try:
    import onnx
    import onnxruntime as ort
except ImportError:
    print("错误: 请先安装依赖:")
    print("  pip install onnx onnxruntime")
    sys.exit(1)


def read_tokens_txt(tokens_path):
    """读取 tokens.txt，返回 token 列表和 vocab_size"""
    tokens = []
    with open(tokens_path, 'r', encoding='utf-8') as f:
        for line in f:
            token = line.strip()
            if token:
                tokens.append(token)
    return tokens, len(tokens)


def read_am_mvn(am_mvn_path):
    """读取 am.mvn 文件，返回 neg_mean 和 inv_stddev"""
    # am.mvn 可能是多种格式：
    # 1. 文本格式：两行，第一行是 mean，第二行是 stddev
    # 2. numpy .npy 格式
    # 3. kaldiio 格式（二进制）
    
    errors = []
    
    # 方法1: 尝试作为文本文件读取
    try:
        with open(am_mvn_path, 'r', encoding='utf-8') as f:
            lines = f.readlines()
            if len(lines) >= 2:
                mean = [float(x) for x in lines[0].strip().split()]
                stddev = [float(x) for x in lines[1].strip().split()]
                if len(mean) > 0 and len(stddev) > 0:
                    neg_mean = [-x for x in mean]
                    inv_stddev = [1.0 / x if x != 0 else 1.0 for x in stddev]
                    return neg_mean, inv_stddev
    except Exception as e:
        errors.append(f"文本格式读取失败: {e}")
    
    # 方法2: 尝试作为 numpy 二进制文件读取 (.npy)
    try:
        data = np.load(am_mvn_path, allow_pickle=False)
        if isinstance(data, np.ndarray):
            if data.shape[0] >= 2:
                mean = data[0].tolist()
                stddev = data[1].tolist()
                neg_mean = [-x for x in mean]
                inv_stddev = [1.0 / x if x != 0 else 1.0 for x in stddev]
                return neg_mean, inv_stddev
            else:
                errors.append(f"numpy 数组行数不足: {data.shape[0]} < 2")
    except Exception as e:
        errors.append(f"numpy 格式读取失败: {e}")
    
    # 方法3: 尝试使用 kaldiio 格式（如果安装了）
    try:
        import kaldiio
        data = kaldiio.load_mat(am_mvn_path)
        if data.shape[0] >= 2:
            mean = data[0].tolist()
            stddev = data[1].tolist()
            neg_mean = [-x for x in mean]
            inv_stddev = [1.0 / x if x != 0 else 1.0 for x in stddev]
            return neg_mean, inv_stddev
        else:
            errors.append(f"kaldiio 数组行数不足: {data.shape[0]} < 2")
    except ImportError:
        errors.append("kaldiio 未安装，跳过 kaldiio 格式尝试")
    except Exception as e:
        errors.append(f"kaldiio 格式读取失败: {e}")
    
    # 方法4: 尝试作为二进制文件读取（可能是 Kaldi 二进制格式）
    try:
        with open(am_mvn_path, 'rb') as f:
            # 读取文件头，检查是否是 Kaldi 格式
            header = f.read(4)
            if header == b'\x00B\x00C':  # Kaldi binary format marker
                # 这是一个简化的 Kaldi 读取，实际可能需要更复杂的解析
                errors.append("检测到 Kaldi 二进制格式，需要 kaldiio 或 scipy.io 来读取")
    except Exception as e:
        errors.append(f"二进制格式检查失败: {e}")
    
    error_msg = f"无法读取 am.mvn 文件: {am_mvn_path}\n尝试的方法:\n" + "\n".join(f"  - {e}" for e in errors)
    raise ValueError(error_msg)


def read_config_yaml(config_path):
    """读取 config.yaml，提取 lfr_window_size 和 lfr_window_shift"""
    with open(config_path, 'r', encoding='utf-8') as f:
        config = yaml.safe_load(f)
    
    # 尝试从不同路径获取 LFR 参数
    lfr_window_size = None
    lfr_window_shift = None
    
    # 常见路径
    paths_to_check = [
        ['frontend_conf', 'lfr_m'],
        ['frontend_conf', 'lfr_n'],
        ['frontend_conf', 'lfr_window_size'],
        ['frontend_conf', 'lfr_window_shift'],
        ['model_conf', 'lfr_m'],
        ['model_conf', 'lfr_n'],
    ]
    
    for path in paths_to_check:
        value = config
        try:
            for key in path:
                value = value[key]
            if 'window_size' in path[-1] or 'lfr_m' in path[-1]:
                lfr_window_size = int(value)
            elif 'window_shift' in path[-1] or 'lfr_n' in path[-1]:
                lfr_window_shift = int(value)
        except (KeyError, TypeError):
            continue
    
    # 如果没找到，使用默认值
    if lfr_window_size is None:
        lfr_window_size = 7  # Paraformer 默认值
    if lfr_window_shift is None:
        lfr_window_shift = 6  # Paraformer 默认值
    
    return lfr_window_size, lfr_window_shift


def check_metadata_exists(model_path):
    """检查模型是否已有必要的元数据"""
    try:
        model = onnx.load(str(model_path))
        metadata = {prop.key: prop.value for prop in model.metadata_props}
        
        required_keys = ['vocab_size', 'lfr_window_size', 'lfr_window_shift', 'neg_mean', 'inv_stddev']
        missing_keys = [key for key in required_keys if key not in metadata]
        
        return len(missing_keys) == 0, missing_keys
    except Exception as e:
        print(f"警告: 无法检查元数据: {e}")
        return False, []


def add_metadata_to_model(model_dir, model_path=None):
    """为模型添加元数据
    
    Args:
        model_dir: 源目录，包含 tokens.txt、am.mvn、config.yaml 等文件
        model_path: 目标模型文件路径（.onnx），如果为 None 则在 model_dir 中查找 model.onnx
                    如果传入的是目录，则在该目录中查找 model.onnx
    """
    model_dir = Path(model_dir)
    
    # 确定目标模型文件路径
    if model_path is None:
        model_path = model_dir / "model.onnx"
    else:
        model_path = Path(model_path)
        # 如果 model_path 是目录，则在其中查找 model.onnx
        if model_path.is_dir():
            potential_model = model_path / "model.onnx"
            if potential_model.exists():
                model_path = potential_model
            else:
                # 如果目标目录中没有 model.onnx，尝试从源目录复制
                source_model = model_dir / "model.onnx"
                if source_model.exists():
                    print(f"目标目录中没有 model.onnx，从源目录复制...")
                    import shutil
                    shutil.copy2(source_model, potential_model)
                    print(f"✓ 已复制 model.onnx 到 {potential_model}")
                    model_path = potential_model
                else:
                    # 尝试查找源目录中的任何 .onnx 文件
                    onnx_files = list(model_dir.glob("*.onnx"))
                    if onnx_files:
                        source_model = onnx_files[0]
                        print(f"目标目录中没有 model.onnx，从源目录复制 {source_model.name}...")
                        import shutil
                        shutil.copy2(source_model, potential_model)
                        print(f"✓ 已复制 {source_model.name} 到 {potential_model}")
                        model_path = potential_model
                    else:
                        raise FileNotFoundError(
                            f"在目录 {model_path} 中未找到 model.onnx，"
                            f"且源目录 {model_dir} 中也没有找到 .onnx 文件"
                        )
    
    # 确定模型所在目录（用于查找其他文件，作为备选）
    model_file_dir = model_path.parent
    
    # 查找必要文件（优先在 model_dir 中查找，如果不存在则在 model_file_dir 中查找）
    tokens_path = model_dir / "tokens.txt"
    if not tokens_path.exists():
        tokens_path = model_file_dir / "tokens.txt"
    
    am_mvn_path = model_dir / "am.mvn"
    if not am_mvn_path.exists():
        am_mvn_path = model_file_dir / "am.mvn"
    
    config_path = model_dir / "config.yaml"
    if not config_path.exists():
        config_path = model_file_dir / "config.yaml"
    
    # 检查文件是否存在
    if not model_path.exists():
        raise FileNotFoundError(f"模型文件不存在: {model_path}")
    if not tokens_path.exists():
        raise FileNotFoundError(f"tokens.txt 不存在: 已检查 {model_dir} 和 {model_file_dir}")
    if not am_mvn_path.exists():
        raise FileNotFoundError(f"am.mvn 不存在: 已检查 {model_dir} 和 {model_file_dir}")
    
    # 检查是否已有元数据
    has_metadata, missing_keys = check_metadata_exists(model_path)
    if has_metadata:
        print(f"✓ 模型已有完整的元数据，无需添加")
        return False
    
    print(f"模型缺少以下元数据: {', '.join(missing_keys)}")
    print("开始添加元数据...")
    
    # 读取必要信息
    print(f"读取 tokens.txt...")
    tokens, vocab_size = read_tokens_txt(tokens_path)
    print(f"  vocab_size = {vocab_size}")
    
    print(f"读取 am.mvn...")
    neg_mean, inv_stddev = read_am_mvn(am_mvn_path)
    print(f"  neg_mean 长度 = {len(neg_mean)}")
    print(f"  inv_stddev 长度 = {len(inv_stddev)}")
    
    lfr_window_size = 7  # 默认值
    lfr_window_shift = 6  # 默认值
    
    if config_path.exists():
        print(f"读取 config.yaml...")
        try:
            lfr_window_size, lfr_window_shift = read_config_yaml(config_path)
            print(f"  lfr_window_size = {lfr_window_size}")
            print(f"  lfr_window_shift = {lfr_window_shift}")
        except Exception as e:
            print(f"  警告: 无法从 config.yaml 读取 LFR 参数，使用默认值: {e}")
    
    # 加载模型
    print(f"加载 ONNX 模型...")
    model = onnx.load(str(model_path))
    
    # 添加元数据
    print("添加元数据到模型...")
    metadata_props = [
        ('vocab_size', str(vocab_size)),
        ('lfr_window_size', str(lfr_window_size)),
        ('lfr_window_shift', str(lfr_window_shift)),
        ('neg_mean', ','.join(map(str, neg_mean))),
        ('inv_stddev', ','.join(map(str, inv_stddev))),
    ]
    
    # 清除现有元数据（如果需要）
    existing_keys = {prop.key for prop in model.metadata_props}
    for key, value in metadata_props:
        if key in existing_keys:
            # 移除旧的
            model.metadata_props[:] = [p for p in model.metadata_props if p.key != key]
        # 添加新的
        model.metadata_props.append(onnx.StringStringEntryProto(key=key, value=value))
    
    # 保存模型
    print(f"保存模型到 {model_path}...")
    onnx.save(model, str(model_path))
    
    print("✓ 元数据添加完成！")
    return True


def main():
    if len(sys.argv) < 2:
        print("用法: python add_paraformer_metadata.py <source_dir> [target_model_or_dir]")
        print("")
        print("参数:")
        print("  source_dir: 源目录，包含 tokens.txt、am.mvn、config.yaml 等文件")
        print("  target_model_or_dir: 目标模型文件路径（.onnx）或目录（在其中查找 model.onnx）")
        print("                       如果省略，则在 source_dir 中查找 model.onnx")
        print("")
        print("示例:")
        print("  # 源目录和目标目录相同")
        print("  python add_paraformer_metadata.py /path/to/model")
        print("")
        print("  # 从源目录读取文件，添加到目标目录的模型")
        print("  python add_paraformer_metadata.py /path/to/source /path/to/target/model.onnx")
        print("  python add_paraformer_metadata.py /path/to/source /path/to/target_dir")
        sys.exit(1)
    
    model_dir = Path(sys.argv[1]).expanduser().resolve()
    model_path = Path(sys.argv[2]).expanduser().resolve() if len(sys.argv) > 2 else None
    
    if not model_dir.exists():
        print(f"错误: 模型目录不存在: {model_dir}")
        sys.exit(1)
    
    # 如果 model_path 是目录，打印提示信息
    if model_path and model_path.is_dir():
        print(f"注意: 第二个参数是目录，将在其中查找 model.onnx")
        print(f"  模型目录: {model_dir}")
        print(f"  输出目录: {model_path}")
    
    try:
        add_metadata_to_model(model_dir, model_path)
    except Exception as e:
        print(f"错误: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
