import os
import urllib.request
import json
import sys

def get_env_var(key, default=None):
    val = os.getenv(key)
    if val:
        return val
    # Fallback to reading .env file locally if exists
    env_file = os.path.join(os.path.dirname(os.path.dirname(__file__)), ".env")
    if os.path.exists(env_file):
        with open(env_file, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    if k.strip() == key:
                        return v.strip()
    return default

NOTION_TOKEN = get_env_var("NOTION_TOKEN")
DATABASE_ID = get_env_var("NOTION_DATABASE_ID", "4035f969-eb53-8196-ae61-cbf2c1f95411")

HEADERS = {
    "Authorization": f"Bearer {NOTION_TOKEN}",
    "Notion-Version": "2022-06-28",
    "Content-Type": "application/json"
}

def query_tasks(search_term=None):
    url = f"https://api.notion.com/v1/databases/{DATABASE_ID}/query"
    payload = {}
    if search_term:
        payload = {
            "filter": {
                "or": [
                    {
                        "property": "Tarefa",
                        "title": {
                            "contains": search_term
                        }
                    },
                    {
                        "property": "ID",
                        "rich_text": {
                            "contains": search_term
                        }
                    }
                ]
            }
        }
    req = urllib.request.Request(url, headers=HEADERS, data=json.dumps(payload).encode('utf-8'))
    with urllib.request.urlopen(req) as resp:
        data = json.loads(resp.read().decode())
        results = []
        for p in data.get("results", []):
            page_id = p["id"]
            props = p.get("properties", {})
            title = props.get("Tarefa", {}).get("title", [{}])[0].get("plain_text", "Sem Título")
            tid = props.get("ID", {}).get("rich_text", [{}])[0].get("plain_text", "N/A")
            status = props.get("Status", {}).get("status", {}).get("name", "N/A")
            phase = props.get("Fase", {}).get("select", {}).get("name", "N/A")
            results.append({"page_id": page_id, "id": tid, "title": title, "status": status, "phase": phase})
        return results

def update_task_status(page_id, new_status, title=None):
    url = f"https://api.notion.com/v1/pages/{page_id}"
    props = {
        "Status": {
            "status": {"name": new_status}
        }
    }
    if title:
        props["Tarefa"] = {
            "title": [{"text": {"content": title}}]
        }
    payload = {
        "properties": props
    }
    req = urllib.request.Request(url, headers=HEADERS, data=json.dumps(payload).encode('utf-8'), method="PATCH")
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())

def create_task(task_id, title, phase, priority, deps, description):
    url = "https://api.notion.com/v1/pages"
    payload = {
        "parent": {
            "database_id": DATABASE_ID
        },
        "properties": {
            "Tarefa": {
                "title": [{"text": {"content": title}}]
            },
            "ID": {
                "rich_text": [{"text": {"content": task_id}}]
            },
            "Fase": {
                "select": {"name": phase}
            },
            "Status": {
                "status": {"name": "Not started"}
            },
            "Prioridade": {
                "select": {"name": priority}
            },
            "Dependências": {
                "rich_text": [{"text": {"content": deps}}]
            }
        },
        "children": [
            {
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{"type": "text", "text": {"content": description}}]
                }
            }
        ]
    }
    req = urllib.request.Request(url, headers=HEADERS, data=json.dumps(payload).encode('utf-8'))
    with urllib.request.urlopen(req) as resp:
        return json.loads(resp.read().decode())["id"]

if __name__ == "__main__":
    if len(sys.argv) > 1:
        cmd = sys.argv[1]
        if cmd == "search" and len(sys.argv) > 2:
            items = query_tasks(sys.argv[2])
            print(json.dumps(items, indent=2, ensure_ascii=False))
        elif cmd == "update" and len(sys.argv) > 3:
            target_id = sys.argv[2]
            new_status = sys.argv[3]
            new_title = sys.argv[4] if len(sys.argv) > 4 else None
            items = query_tasks(target_id)
            matched = [it for it in items if it["id"].strip().upper() == target_id.strip().upper()]
            if not matched:
                print(f"Task with ID {target_id} not found.")
            else:
                for it in matched:
                    res = update_task_status(it["page_id"], new_status, new_title)
                    print(f"Updated {it['id']} to status '{new_status}' (title: {new_title or it['title']}).")
        elif cmd == "create" and len(sys.argv) > 7:
            tid = sys.argv[2]
            title = sys.argv[3]
            phase = sys.argv[4]
            priority = sys.argv[5]
            deps = sys.argv[6]
            desc = sys.argv[7]
            pid = create_task(tid, title, phase, priority, deps, desc)
            print(f"Created task {tid} in Notion with Page ID: {pid}")
        elif cmd == "list":
            items = query_tasks()
            print(f"Total tasks: {len(items)}")
            for item in items:
                print(f"[{item['id']}] {item['title']} - {item['status']}")
    else:
        print("Usage: python notion_sync.py [search <term> | list | update <id> <status>]")
