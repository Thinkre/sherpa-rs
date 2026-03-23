import os
import dashscope
import json
import time
import random
from tqdm import tqdm

# Replace ABSOLUTE_PATH/welcome.mp3 with the absolute path of your local audio file.
audio_file_path = "file:///mnt/bella/users/zhaoshuaijiang/data/test_vrdk_beijing_jingjiren/wavs/out-audio-linenum-500538794-1000000020197253-f-1597923302333-t-1597924024447_10.wav"

api_key = 'sk-3ad177abab024cb19e9ca794b8a86081'

def read_data(inp):
    if inp.endswith('.wav'):
        audio_file_path = f"file://{inp}"
        key = 'audio'
        data = [key, audio_file_path]
        data = [{
            'key': key,
            'wav': audio_file_path
        }]
    elif inp.endswith('.json') or inp.endswith('.jsonl'):
        data = []
        for line in open(inp, 'r'):
            d = json.loads(line)
            data.append(d)
    else:
        data = []
        for line in open(inp, 'r'):
            key, audio_file_path = line.strip().rsplit(maxsplit=1)
            data.append({
                'key': key,
                'wav': audio_file_path
            })
    return data

def get_processed_keys(out):
    if os.path.exists(out):
        keys = set()
        for line in open(out, 'r'):
            key = line.strip().rsplit(maxsplit=1)[0]
            keys.add(key)
        return keys
    else:
        return set()

def make_request_with_retry(api_key, model, messages, max_retries=3, base_delay=1):
    """
    带重试机制的API请求函数
    """
    for attempt in range(max_retries):
        try:
            response = dashscope.MultiModalConversation.call(
                api_key=api_key,
                model=model,
                messages=messages,
                result_format="message",
                asr_options={
                    "language": "zh",
                    "enable_lid": True,
                    "enable_itn": False
                }
            )
            
            # 如果请求成功，直接返回
            if response['status_code'] == 200:
                return response
            
            # 如果是429错误（请求频率限制），等待后重试
            elif response['status_code'] == 429:
                if attempt < max_retries - 1:
                    # 指数退避策略，加上随机抖动
                    delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
                    print(f"请求频率限制，等待 {delay:.2f} 秒后重试 (第 {attempt + 1} 次重试)")
                    time.sleep(delay)
                    continue
                else:
                    print(f"重试 {max_retries} 次后仍然失败: {response}")
                    return response
            
            # 其他错误直接返回
            else:
                print(f"请求失败: {response}")
                return response
                
        except Exception as e:
            if attempt < max_retries - 1:
                delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
                print(f"请求异常: {e}，等待 {delay:.2f} 秒后重试 (第 {attempt + 1} 次重试)")
                time.sleep(delay)
                continue
            else:
                print(f"重试 {max_retries} 次后仍然异常: {e}")
                raise e
    
    return None

def infer(
    inp,
    out = None,
    model = "qwen3-asr-flash",
    system_prompt = "",
    request_interval = 0.8,  # 请求间隔时间（秒）
    ):
    data = read_data(inp)

    if out is None:
        data_dir = os.path.dirname(inp)
        out_dir = os.path.join(data_dir, model)
        out = os.path.join(out_dir, 'text')
        os.makedirs(out_dir, exist_ok=True)

    processed_keys = get_processed_keys(out)

    fw = open(out, 'a+')
    res = []
    
    for i, d in enumerate(tqdm(data, desc="处理音频文件")):
        if d['key'] in processed_keys:
            continue

        # 添加请求间隔，避免请求过于频繁
        if i > 0:
            time.sleep(request_interval)
        
        messages = [
            {
                "role": "system",
                "content": [
                    {"text": system_prompt},
                ]
            },
            {
                "role": "user",
                "content": [
                    {"audio": d['wav']},
                ]
            }
        ]
        
        # 使用带重试机制的请求函数
        
        response = make_request_with_retry(api_key, model, messages)
        
        if response and response['status_code'] == 200:
            d[model] = response['output']['choices'][0]['message']['content'][0]['text']
            fw.write(f'{d["key"]}\t{d[model]}\n')
            res.append(d)
        else:
            print(f"处理失败: {d['key']} - {response}")

    fw.close()
    return res

if __name__ == '__main__':
    from fire import Fire
    Fire(infer)