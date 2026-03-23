#!/usr/bin/env python3
"""
验证特征提取 - 对比FunASR和Rust实现的特征是否一致
"""

import sys
import numpy as np
import librosa

# 添加FunASR路径
sys.path.insert(0, "/Users/thinkre/Desktop/open/FunASR_2601/runtime/python/onnxruntime")

from funasr_onnx.utils.frontend import WavFrontend

def extract_funasr_features(audio_file, model_dir):
    """使用FunASR提取特征"""

    # 加载音频
    audio, sr = librosa.load(audio_file, sr=16000)
    print(f"Audio loaded: length={len(audio)}, sr={sr}")
    print(f"Audio samples (first 5): {audio[:5]}")

    # 创建frontend
    cmvn_file = f"{model_dir}/am.mvn"
    frontend = WavFrontend(
        cmvn_file=cmvn_file,
        fs=16000,
        window="hamming",
        n_mels=80,
        frame_length=25,
        frame_shift=10,
        lfr_m=7,
        lfr_n=6,
        dither=1.0,  # 注意：默认是1.0
    )

    # 提取FBank
    print("\n=== Step 1: FBank ===")
    fbank_feat, fbank_len = frontend.fbank(audio)
    print(f"FBank shape: {fbank_feat.shape}, length: {fbank_len}")
    print(f"FBank[0, :10]: {fbank_feat[0, :10]}")
    print(f"FBank[0, :10] (raw): {fbank_feat[0, :10].tolist()}")

    # 应用LFR + CMVN
    print("\n=== Step 2: LFR + CMVN ===")
    feat, feat_len = frontend.lfr_cmvn(fbank_feat)
    print(f"After LFR+CMVN shape: {feat.shape}, length: {feat_len}")
    print(f"After LFR+CMVN[0, :10]: {feat[0, :10]}")
    print(f"After LFR+CMVN[0, :10] (raw): {feat[0, :10].tolist()}")

    return feat, feat_len

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python verify_features.py <audio_file> <model_dir>")
        print("Example: python verify_features.py test.wav /path/to/seaco-paraformer-model")
        sys.exit(1)

    audio_file = sys.argv[1]
    model_dir = sys.argv[2]

    print(f"Extracting features from: {audio_file}")
    print(f"Using model dir: {model_dir}")
    print("=" * 60)

    feat, feat_len = extract_funasr_features(audio_file, model_dir)

    print("\n" + "=" * 60)
    print("对比步骤：")
    print("1. 运行Rust转录，查看日志中的特征值")
    print("2. 对比以下数值：")
    print(f"   - FBank[0, :5]")
    print(f"   - After LFR+CMVN[0, :5]")
    print("3. 如果数值差异很大，则确认是特征提取问题")
