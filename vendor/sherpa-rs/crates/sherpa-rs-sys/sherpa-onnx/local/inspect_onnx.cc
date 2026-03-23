// Simple tool to inspect ONNX model structure
#include <iostream>
#include <string>

#include "sherpa-onnx/csrc/onnx-utils.h"
#include "sherpa-onnx/csrc/session.h"
#include "onnxruntime_cxx_api.h"

int main(int argc, char *argv[]) {
  if (argc != 2) {
    std::cerr << "Usage: " << argv[0] << " <model.onnx>\n";
    return 1;
  }

  std::string model_path = argv[1];
  
  Ort::Env env(ORT_LOGGING_LEVEL_WARNING);
  Ort::SessionOptions sess_opts;
  sess_opts.SetIntraOpNumThreads(1);
  
  auto sess = std::make_unique<Ort::Session>(env, model_path.c_str(), sess_opts);
  
  // Get metadata
  Ort::ModelMetadata meta_data = sess->GetModelMetadata();
  Ort::AllocatorWithDefaultOptions allocator;
  
  std::cout << "=== Model Metadata ===\n";
  PrintModelMetadata(std::cout, meta_data);
  
  // Get input/output names
  std::vector<std::string> input_names;
  std::vector<const char*> input_names_ptr;
  GetInputNames(sess.get(), &input_names, &input_names_ptr);
  
  std::cout << "\n=== Inputs ===\n";
  for (size_t i = 0; i < input_names.size(); ++i) {
    auto type_info = sess->GetInputTypeInfo(i);
    auto tensor_info = type_info.GetTensorTypeAndShapeInfo();
    auto shape = tensor_info.GetShape();
    std::cout << i << ": " << input_names[i] << " ";
    std::cout << "type=" << tensor_info.GetElementType() << " shape=[";
    for (size_t j = 0; j < shape.size(); ++j) {
      if (j > 0) std::cout << ", ";
      if (shape[j] == -1) {
        std::cout << "?";
      } else {
        std::cout << shape[j];
      }
    }
    std::cout << "]\n";
  }
  
  std::vector<std::string> output_names;
  std::vector<const char*> output_names_ptr;
  GetOutputNames(sess.get(), &output_names, &output_names_ptr);
  
  std::cout << "\n=== Outputs ===\n";
  for (size_t i = 0; i < output_names.size(); ++i) {
    auto type_info = sess->GetOutputTypeInfo(i);
    auto tensor_info = type_info.GetTensorTypeAndShapeInfo();
    auto shape = tensor_info.GetShape();
    std::cout << i << ": " << output_names[i] << " ";
    std::cout << "type=" << tensor_info.GetElementType() << " shape=[";
    for (size_t j = 0; j < shape.size(); ++j) {
      if (j > 0) std::cout << ", ";
      if (shape[j] == -1) {
        std::cout << "?";
      } else {
        std::cout << shape[j];
      }
    }
    std::cout << "]\n";
  }
  
  return 0;
}
