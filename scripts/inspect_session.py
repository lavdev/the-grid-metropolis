import json

latest = r'C:\Users\Lav\.codex\sessions\2026\10\28\rollout-2026-10-28T18-01-00-01a18450-f26c-70e1-aa00-b71941a9f399.jsonl'
with open(latest, 'r', encoding='utf-8') as f:
    for i, line in enumerate(f):
        data = json.loads(line)
        t = data.get('type')
        payload = data.get('payload', {})
        pt = payload.get('type')
        print(f"[{i}] type={t} | payload.type={pt}")
        if 'content' in payload:
            print(f"   content: {str(payload['content'])[:120]}")
        if 'text' in payload:
            print(f"   text: {str(payload['text'])[:120]}")
        if 'tool_call' in str(data):
            print(f"   TOOL: {str(data)[:250]}")
