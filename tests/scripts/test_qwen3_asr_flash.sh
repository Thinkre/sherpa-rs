# ======= 重要提示 =======
# 以下为北京地域url，若使用新加坡地域的模型，需将url替换为：https://dashscope-intl.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation，若使用美国地域的模型，需将url替换为：https://dashscope-us.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation
# 新加坡/美国地域和北京地域的API Key不同。获取API Key：https://help.aliyun.com/zh/model-studio/get-api-key
# 若使用美国地域的模型，需要加us后缀
# === 执行时请删除该注释 ===

curl -X POST "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation" \
-H "Authorization: Bearer sk-3ad177abab024cb19e9ca794b8a86081" \
-H "Content-Type: application/json" \
-d '{
    "model": "qwen3-asr-flash",
    "input": {
        "messages": [
            {
                "content": [
                    {
                        "text": ""
                    }
                ],
                "role": "system"
            },
            {
                "content": [
                    {
                        "audio": "/Users/thinkre/Desktop/projects/KeVoiceInput/scripts/test.wav"
                    }
                ],
                "role": "user"
            }
        ]
    },
    "parameters": {
        "asr_options": {
            "enable_itn": false
        }
    }
}'
