// test_seaco_paraformer_c_api.c
//
// Test SeACo-Paraformer with C-API
//
// Compile:
//   gcc -o test_seaco_paraformer_c_api test_seaco_paraformer_c_api.c \
//       -I./sherpa-onnx/c-api -L./build/lib -lsherpa-onnx-c-api \
//       -Wl,-rpath,./build/lib
//
// Run:
//   ./test_seaco_paraformer_c_api <wav_file>

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "sherpa-onnx/c-api/c-api.h"

int32_t main(int32_t argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Usage: %s <wav_file> [hotwords_file]\n", argv[0]);
    fprintf(stderr, "\nExample:\n");
    fprintf(stderr, "  %s test.wav\n", argv[0]);
    fprintf(stderr, "  %s test.wav hotwords.txt\n", argv[0]);
    return -1;
  }

  const char *wav_filename = argv[1];
  const char *hotwords_file = (argc >= 3) ? argv[2] : NULL;

  // Model paths
  const char *model_dir = 
      "/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.sherpa_onnx";
  const char *model_filename = 
      "/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.sherpa_onnx/model.onnx";
  const char *model_eb_filename = 
      "/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.sherpa_onnx/model_eb.onnx";
  const char *tokens_filename = 
      "/Users/thinkre/Desktop/models/beike/seaco_paraformer.20250904.for_general.sherpa_onnx/tokens.txt";
  const char *provider = "cpu";

  fprintf(stderr, "Reading wave file: %s\n", wav_filename);
  const SherpaOnnxWave *wave = SherpaOnnxReadWave(wav_filename);
  if (wave == NULL) {
    fprintf(stderr, "Failed to read %s\n", wav_filename);
    return -1;
  }

  fprintf(stderr, "Wave file info:\n");
  fprintf(stderr, "  Sample rate: %d Hz\n", wave->sample_rate);
  fprintf(stderr, "  Num samples: %d\n", wave->num_samples);
  fprintf(stderr, "  Duration: %.2f seconds\n", 
          (float)wave->num_samples / wave->sample_rate);

  // SeACo-Paraformer config
  fprintf(stderr, "\nConfiguring SeACo-Paraformer model...\n");
  SherpaOnnxOfflineParaformerModelConfig paraformer_config;
  memset(&paraformer_config, 0, sizeof(paraformer_config));
  paraformer_config.model = model_filename;
  paraformer_config.model_eb = model_eb_filename;  // SeACo-Paraformer embedding model

  fprintf(stderr, "  Model: %s\n", paraformer_config.model);
  fprintf(stderr, "  Model EB: %s\n", paraformer_config.model_eb);

  // Offline model config
  SherpaOnnxOfflineModelConfig offline_model_config;
  memset(&offline_model_config, 0, sizeof(offline_model_config));
  offline_model_config.debug = 1;
  offline_model_config.num_threads = 2;
  offline_model_config.provider = provider;
  offline_model_config.tokens = tokens_filename;
  offline_model_config.paraformer = paraformer_config;

  // Recognizer config
  SherpaOnnxOfflineRecognizerConfig recognizer_config;
  memset(&recognizer_config, 0, sizeof(recognizer_config));
  recognizer_config.decoding_method = "greedy_search";
  recognizer_config.model_config = offline_model_config;
  
  if (hotwords_file) {
    recognizer_config.hotwords_file = hotwords_file;
    fprintf(stderr, "  Hotwords file: %s\n", hotwords_file);
  } else {
    fprintf(stderr, "  Hotwords file: (not provided)\n");
  }

  fprintf(stderr, "\nCreating recognizer...\n");
  const SherpaOnnxOfflineRecognizer *recognizer =
      SherpaOnnxCreateOfflineRecognizer(&recognizer_config);

  if (recognizer == NULL) {
    fprintf(stderr, "Failed to create recognizer! Please check your config!\n");
    SherpaOnnxFreeWave(wave);
    return -1;
  }

  fprintf(stderr, "Recognizer created successfully!\n");

  fprintf(stderr, "\nCreating stream...\n");
  const SherpaOnnxOfflineStream *stream =
      SherpaOnnxCreateOfflineStream(recognizer);

  fprintf(stderr, "Accepting waveform...\n");
  SherpaOnnxAcceptWaveformOffline(stream, wave->sample_rate, wave->samples,
                                  wave->num_samples);

  fprintf(stderr, "Decoding...\n");
  SherpaOnnxDecodeOfflineStream(recognizer, stream);
  
  fprintf(stderr, "Getting result...\n");
  const SherpaOnnxOfflineRecognizerResult *result =
      SherpaOnnxGetOfflineStreamResult(stream);

  fprintf(stderr, "\n");
  fprintf(stderr, "========================================\n");
  fprintf(stderr, "Recognition Result:\n");
  fprintf(stderr, "========================================\n");
  fprintf(stderr, "Text: %s\n", result->text);
  
  if (result->timestamps && result->count > 0) {
    fprintf(stderr, "Timestamps (%d): ", result->count);
    for (int32_t i = 0; i < result->count; ++i) {
      fprintf(stderr, "%.2f ", result->timestamps[i]);
    }
    fprintf(stderr, "\n");
  } else {
    fprintf(stderr, "Timestamps: (not available)\n");
  }
  fprintf(stderr, "========================================\n");

  // Cleanup
  SherpaOnnxDestroyOfflineRecognizerResult(result);
  SherpaOnnxDestroyOfflineStream(stream);
  SherpaOnnxDestroyOfflineRecognizer(recognizer);
  SherpaOnnxFreeWave(wave);

  fprintf(stderr, "\nTest completed successfully!\n");
  return 0;
}
