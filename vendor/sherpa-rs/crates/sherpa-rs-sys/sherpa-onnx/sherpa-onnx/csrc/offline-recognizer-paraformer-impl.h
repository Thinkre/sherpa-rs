// sherpa-onnx/csrc/offline-recognizer-paraformer-impl.h
//
// Copyright (c)  2022-2023  Xiaomi Corporation

#ifndef SHERPA_ONNX_CSRC_OFFLINE_RECOGNIZER_PARAFORMER_IMPL_H_
#define SHERPA_ONNX_CSRC_OFFLINE_RECOGNIZER_PARAFORMER_IMPL_H_

#include <algorithm>
#include <fstream>
#include <memory>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

#include "Eigen/Dense"
#include "sherpa-onnx/csrc/file-utils.h"
#include "sherpa-onnx/csrc/macros.h"
#include "sherpa-onnx/csrc/offline-model-config.h"
#include "sherpa-onnx/csrc/offline-paraformer-decoder.h"
#include "sherpa-onnx/csrc/offline-paraformer-greedy-search-decoder.h"
#include "sherpa-onnx/csrc/offline-paraformer-model.h"
#include "sherpa-onnx/csrc/offline-recognizer-impl.h"
#include "sherpa-onnx/csrc/offline-recognizer.h"
#include "sherpa-onnx/csrc/onnx-utils.h"
#include "sherpa-onnx/csrc/pad-sequence.h"
#include "sherpa-onnx/csrc/symbol-table.h"
#include "sherpa-onnx/csrc/text-utils.h"

namespace sherpa_onnx {

OfflineRecognitionResult Convert(const OfflineParaformerDecoderResult &src,
                                 const SymbolTable &sym_table) {
  OfflineRecognitionResult r;
  r.tokens.reserve(src.tokens.size());
  r.timestamps = src.timestamps;

  std::string text;

  // When the current token ends with "@@" we set mergeable to true
  bool mergeable = false;

  for (int32_t i = 0; i != src.tokens.size(); ++i) {
    auto sym = sym_table[src.tokens[i]];
    r.tokens.push_back(sym);

    if ((sym.back() != '@') || (sym.size() > 2 && sym[sym.size() - 2] != '@')) {
      // sym does not end with "@@"
      const uint8_t *p = reinterpret_cast<const uint8_t *>(sym.c_str());
      if (p[0] < 0x80) {
        // an ascii
        if (mergeable) {
          mergeable = false;
          text.append(sym);
        } else {
          text.append(" ");
          text.append(sym);
        }
      } else {
        // not an ascii
        mergeable = false;

        if (i > 0) {
          const uint8_t p = reinterpret_cast<const uint8_t *>(
              sym_table[src.tokens[i - 1]].c_str())[0];
          if (p < 0x80) {
            // put a space between ascii and non-ascii
            text.append(" ");
          }
        }
        text.append(sym);
      }
    } else {
      // this sym ends with @@
      sym = std::string(sym.data(), sym.size() - 2);
      if (mergeable) {
        text.append(sym);
      } else {
        text.append(" ");
        text.append(sym);
        mergeable = true;
      }
    }
  }
  r.text = std::move(text);

  return r;
}

class OfflineRecognizerParaformerImpl : public OfflineRecognizerImpl {
 public:
  explicit OfflineRecognizerParaformerImpl(
      const OfflineRecognizerConfig &config)
      : OfflineRecognizerImpl(config),
        config_(config),
        symbol_table_(config_.model_config.tokens),
        model_(std::make_unique<OfflineParaformerModel>(config.model_config)) {
    if (config.decoding_method == "greedy_search") {
      int32_t eos_id = symbol_table_["</s>"];
      decoder_ = std::make_unique<OfflineParaformerGreedySearchDecoder>(eos_id);
    } else {
      SHERPA_ONNX_LOGE("Only greedy_search is supported at present. Given %s",
                       config.decoding_method.c_str());
      SHERPA_ONNX_EXIT(-1);
    }

    InitFeatConfig();
    InitHotwords();
  }

  template <typename Manager>
  OfflineRecognizerParaformerImpl(Manager *mgr,
                                  const OfflineRecognizerConfig &config)
      : OfflineRecognizerImpl(mgr, config),
        config_(config),
        symbol_table_(mgr, config_.model_config.tokens),
        model_(std::make_unique<OfflineParaformerModel>(mgr,
                                                        config.model_config)) {
    if (config.decoding_method == "greedy_search") {
      int32_t eos_id = symbol_table_["</s>"];
      decoder_ = std::make_unique<OfflineParaformerGreedySearchDecoder>(eos_id);
    } else {
      SHERPA_ONNX_LOGE("Only greedy_search is supported at present. Given %s",
                       config.decoding_method.c_str());
      SHERPA_ONNX_EXIT(-1);
    }

    InitFeatConfig();
    InitHotwordsFromManager(mgr);
  }

  std::unique_ptr<OfflineStream> CreateStream() const override {
    return std::make_unique<OfflineStream>(config_.feat_config);
  }

  void DecodeStreams(OfflineStream **ss, int32_t n) const override {
    // 1. Apply LFR
    // 2. Apply CMVN
    //
    // Please refer to
    // https://static.googleusercontent.com/media/research.google.com/en//pubs/archive/45555.pdf
    // for what LFR means
    //
    // "Lower Frame Rate Neural Network Acoustic Models"
    auto memory_info =
        Ort::MemoryInfo::CreateCpu(OrtDeviceAllocator, OrtMemTypeDefault);

    std::vector<Ort::Value> features;
    features.reserve(n);

    int32_t feat_dim =
        config_.feat_config.feature_dim * model_->LfrWindowSize();

    std::vector<std::vector<float>> features_vec(n);
    std::vector<int32_t> features_length_vec(n);
    for (int32_t i = 0; i != n; ++i) {
      std::vector<float> f = ss[i]->GetFrames();

      f = ApplyLFR(f);
      ApplyCMVN(&f);

      int32_t num_frames = f.size() / feat_dim;
      features_vec[i] = std::move(f);

      features_length_vec[i] = num_frames;

      std::array<int64_t, 2> shape = {num_frames, feat_dim};

      Ort::Value x = Ort::Value::CreateTensor(
          memory_info, features_vec[i].data(), features_vec[i].size(),
          shape.data(), shape.size());
      features.push_back(std::move(x));
    }

    std::vector<const Ort::Value *> features_pointer(n);
    for (int32_t i = 0; i != n; ++i) {
      features_pointer[i] = &features[i];
    }

    std::array<int64_t, 1> features_length_shape = {n};
    Ort::Value x_length = Ort::Value::CreateTensor(
        memory_info, features_length_vec.data(), n,
        features_length_shape.data(), features_length_shape.size());

    // Caution(fangjun): We cannot pad it with log(eps),
    // i.e., -23.025850929940457f
    Ort::Value x = PadSequence(model_->Allocator(), features_pointer, 0);

    std::vector<Ort::Value> t;
    try {
      // Check if we have hotwords and embedding model for SeACo-Paraformer
      if (model_->HasEmbeddingModel() && !hotwords_ids_.empty()) {
        Ort::Value bias_embed = GenerateBiasEmbed(n);
        t = model_->Forward(std::move(x), std::move(x_length),
                           std::move(bias_embed));
      } else {
        t = model_->Forward(std::move(x), std::move(x_length));
      }
    } catch (const Ort::Exception &ex) {
      SHERPA_ONNX_LOGE("\n\nCaught exception:\n\n%s\n\nReturn an empty result",
                       ex.what());
      return;
    }

    std::vector<OfflineParaformerDecoderResult> results;
    if (t.size() == 2) {
      results = decoder_->Decode(std::move(t[0]), std::move(t[1]));
    } else {
      results =
          decoder_->Decode(std::move(t[0]), std::move(t[1]), std::move(t[3]));
    }

    for (int32_t i = 0; i != n; ++i) {
      auto r = Convert(results[i], symbol_table_);
      r.text = ApplyInverseTextNormalization(std::move(r.text));
      r.text = ApplyHomophoneReplacer(std::move(r.text));
      ss[i]->SetResult(r);
    }
  }

  OfflineRecognizerConfig GetConfig() const override { return config_; }

 private:
  void InitFeatConfig() {
    // Paraformer models assume input samples are in the range
    // [-32768, 32767], so we set normalize_samples to false
    config_.feat_config.normalize_samples = false;
    config_.feat_config.window_type = "hamming";
    config_.feat_config.high_freq = 0;
    config_.feat_config.snip_edges = true;
  }

  std::vector<float> ApplyLFR(const std::vector<float> &in) const {
    int32_t lfr_window_size = model_->LfrWindowSize();
    int32_t lfr_window_shift = model_->LfrWindowShift();
    int32_t in_feat_dim = config_.feat_config.feature_dim;

    int32_t in_num_frames = in.size() / in_feat_dim;
    int32_t out_num_frames =
        (in_num_frames - lfr_window_size) / lfr_window_shift + 1;
    int32_t out_feat_dim = in_feat_dim * lfr_window_size;

    std::vector<float> out(out_num_frames * out_feat_dim);

    const float *p_in = in.data();
    float *p_out = out.data();

    for (int32_t i = 0; i != out_num_frames; ++i) {
      std::copy(p_in, p_in + out_feat_dim, p_out);

      p_out += out_feat_dim;
      p_in += lfr_window_shift * in_feat_dim;
    }

    return out;
  }

  void ApplyCMVN(std::vector<float> *v) const {
    const std::vector<float> &neg_mean = model_->NegativeMean();
    const std::vector<float> &inv_stddev = model_->InverseStdDev();
    int32_t dim = static_cast<int32_t>(neg_mean.size());
    int32_t num_frames = static_cast<int32_t>(v->size()) / dim;

    Eigen::Map<
        Eigen::Matrix<float, Eigen::Dynamic, Eigen::Dynamic, Eigen::RowMajor>>
        mat(v->data(), num_frames, dim);

    Eigen::Map<const Eigen::RowVectorXf> neg_mean_vec(neg_mean.data(), dim);
    Eigen::Map<const Eigen::RowVectorXf> inv_stddev_vec(inv_stddev.data(), dim);

    mat.array() = (mat.array().rowwise() + neg_mean_vec.array()).rowwise() *
                  inv_stddev_vec.array();
  }

  // Hotword support for SeACo-Paraformer
  void InitHotwords() {
    if (config_.hotwords_file.empty() || !model_->HasEmbeddingModel()) {
      return;
    }

    std::ifstream is(config_.hotwords_file);
    if (!is) {
      SHERPA_ONNX_LOGE("Failed to open hotwords file: %s",
                       config_.hotwords_file.c_str());
      return;
    }

    EncodeHotwordsForSeaco(is);

    if (config_.model_config.debug && !hotwords_ids_.empty()) {
      SHERPA_ONNX_LOGE("Loaded %d hotwords for SeACo-Paraformer",
                       static_cast<int32_t>(hotwords_ids_.size()));
    }
  }

  template <typename Manager>
  void InitHotwordsFromManager(Manager *mgr) {
    if (config_.hotwords_file.empty() || !model_->HasEmbeddingModel()) {
      return;
    }

    auto buf = ReadFile(mgr, config_.hotwords_file);
    std::istringstream is(std::string(buf.begin(), buf.end()));

    if (!is) {
      SHERPA_ONNX_LOGE("Failed to read hotwords file: %s",
                       config_.hotwords_file.c_str());
      return;
    }

    EncodeHotwordsForSeaco(is);

    if (config_.model_config.debug && !hotwords_ids_.empty()) {
      SHERPA_ONNX_LOGE("Loaded %d hotwords for SeACo-Paraformer",
                       static_cast<int32_t>(hotwords_ids_.size()));
    }
  }

  void EncodeHotwordsForSeaco(std::istream &is) {
    std::string line;
    while (std::getline(is, line)) {
      // Trim whitespace
      size_t start = line.find_first_not_of(" \t\r\n");
      size_t end = line.find_last_not_of(" \t\r\n");

      if (start == std::string::npos) {
        continue;  // Empty line
      }

      line = line.substr(start, end - start + 1);

      if (line.empty() || line[0] == '#') {
        continue;  // Comment or empty line
      }

      // Split into UTF-8 characters
      std::vector<std::string> chars = SplitUtf8(line);
      std::vector<int32_t> ids;

      bool has_unknown = false;
      for (const auto &ch : chars) {
        if (!symbol_table_.Contains(ch)) {
          SHERPA_ONNX_LOGE("Hotword '%s' contains unknown token: %s",
                          line.c_str(), ch.c_str());
          has_unknown = true;
          break;
        }
        ids.push_back(symbol_table_[ch]);
      }

      if (!has_unknown && !ids.empty()) {
        hotwords_ids_.push_back(ids);
      }
    }
  }

  Ort::Value GenerateBiasEmbed(int32_t batch_size) const {
    if (hotwords_ids_.empty() || !model_->HasEmbeddingModel()) {
      // Return empty bias_embed: (batch_size, 0, embedding_dim)
      return CreateEmptyBiasEmbed(batch_size);
    }


    // Following FunASR's ContextualParaformer implementation:
    // 1. For each hotword, create [token_ids] sequence
    // 2. Calculate length for each hotword (len(ids) - 1, minimum 0)
    // 3. Pad sequences to max_len=10
    // 4. Call embedding model
    // 5. Extract embeddings at the last valid position using lengths

    int32_t sos_id = symbol_table_["<s>"];
    const int32_t max_len = 10;
    int32_t num_hotwords = static_cast<int32_t>(hotwords_ids_.size());

    // Pad hotwords to max_len
    std::vector<int32_t> padded_ids(static_cast<size_t>(num_hotwords + 1) * max_len, 0);
    std::vector<int32_t> lengths(num_hotwords + 1);

    for (size_t i = 0; i < hotwords_ids_.size(); ++i) {
      const auto &ids = hotwords_ids_[i];
      // Length is max(0, len(ids) - 1)
      lengths[i] = std::max(0, static_cast<int32_t>(ids.size()) - 1);

      // Copy ids to padded buffer (up to max_len)
      size_t copy_len = std::min(ids.size(), static_cast<size_t>(max_len));
      for (size_t j = 0; j < copy_len; ++j) {
        padded_ids[i * max_len + j] = ids[j];
      }
    }

    // Add <s> token as the last item with length=0
    padded_ids[num_hotwords * max_len] = sos_id;
    lengths[num_hotwords] = 0;

    // Create input tensors for embedding model
    std::array<int64_t, 2> input_shape{num_hotwords + 1, max_len};

    Ort::Value input_ids = Ort::Value::CreateTensor<int32_t>(
        model_->Allocator(), input_shape.data(), input_shape.size());

    // Copy data to the tensor
    int32_t *input_ptr = input_ids.GetTensorMutableData<int32_t>();
    std::copy(padded_ids.begin(), padded_ids.end(), input_ptr);

    // Call embedding model
    auto outputs = model_->ForwardEmbedding(std::move(input_ids));

    if (outputs.empty()) {
      return CreateEmptyBiasEmbed(batch_size);
    }
    // Extract embeddings at the last valid positions
    // Output shape: (num_hotwords+1, max_len, embedding_dim)
    // We need to extract: embeddings[i, lengths[i], :] for each i
    return TransposeAndIndex(outputs[0], lengths, batch_size);
  }

  Ort::Value CreateEmptyBiasEmbed(int32_t batch_size) const {
    int64_t embedding_dim = 512;  // Default for SeACo-Paraformer

    // Try to get actual embedding dimension from model
    if (model_->HasEmbeddingModel()) {
      // This will be determined dynamically, but for empty tensor we use default
      embedding_dim = 512;
    }

    std::array<int64_t, 3> shape{batch_size, 0, embedding_dim};
    return Ort::Value::CreateTensor<float>(model_->Allocator(), shape.data(),
                                          shape.size());
  }

  Ort::Value TransposeAndIndex(const Ort::Value &embeddings,
                               const std::vector<int32_t> &lengths,
                               int32_t batch_size) const {
    // Get input shape
    auto shape = embeddings.GetTensorTypeAndShapeInfo().GetShape();

    if (shape.size() != 3) {
      return CreateEmptyBiasEmbed(batch_size);
    }

    // The embedding model output shape is (T, N, D) where:
    // T = max_len (10)
    // N = num_hotwords + 1 (includes <s> token)
    // D = embedding_dim (512)
    int64_t T = shape[0];  // max_len
    int64_t N = shape[1];  // num_hotwords + 1
    int64_t D = shape[2];  // embedding_dim

    // Get data pointer
    const float *data = embeddings.GetTensorData<float>();

    // Extract embeddings at the last valid position for each hotword
    // Remove the last one (<s> token)
    int64_t num_valid_hotwords = N - 1;
    std::vector<float> result_data(num_valid_hotwords * D);

    for (int64_t i = 0; i < num_valid_hotwords; ++i) {
      int32_t len = lengths[i];
      // Extract embeddings[len, i, :] from (T, N, D) layout
      const float *src = data + len * N * D + i * D;
      float *dst = result_data.data() + i * D;
      std::copy(src, src + D, dst);
    }

    // Create output tensor: (batch_size, num_valid_hotwords, D)
    // All samples in the batch share the same hotwords
    std::array<int64_t, 3> output_shape{batch_size, num_valid_hotwords, D};
    Ort::Value result = Ort::Value::CreateTensor<float>(
        model_->Allocator(), output_shape.data(), output_shape.size());

    float *result_ptr = result.GetTensorMutableData<float>();

    // Replicate the same hotword embeddings for each batch
    for (int32_t b = 0; b < batch_size; ++b) {
      std::copy(result_data.begin(), result_data.end(),
                result_ptr + b * num_valid_hotwords * D);
    }

    return result;
  }

  OfflineRecognizerConfig config_;
  SymbolTable symbol_table_;
  std::unique_ptr<OfflineParaformerModel> model_;
  std::unique_ptr<OfflineParaformerDecoder> decoder_;

  // Hotwords for SeACo-Paraformer
  std::vector<std::vector<int32_t>> hotwords_ids_;
};

}  // namespace sherpa_onnx

#endif  // SHERPA_ONNX_CSRC_OFFLINE_RECOGNIZER_PARAFORMER_IMPL_H_
