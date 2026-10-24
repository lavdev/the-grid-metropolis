import argparse
import json
import os
import sys
import time
import urllib.request
import urllib.error

DEFAULT_HTTP_URL = os.getenv("BRIDGE_HTTP_URL", "http://localhost:3000")
DEFAULT_WS_URL = os.getenv("BRIDGE_WS_URL", "ws://localhost:3000/ws")
DEFAULT_TOKEN = os.getenv("BRIDGE_AUTH_TOKEN", "secret-bridge-token")

def send_dispatch(http_url, token, task_id, title, agent_id, project_id, priority, prompt):
    url = f"{http_url.rstrip('/')}/api/v1/bridge/dispatch"
    payload = {
        "source": "cli-mock-runner",
        "projectId": project_id,
        "targetAgentId": agent_id,
        "task": {
            "taskId": task_id,
            "title": title,
            "priority": priority,
            "payload": {
                "prompt": prompt,
                "contextFiles": ["src/main.rs"]
            }
        },
        "callbackWebhook": f"https://mock.engine.io/tasks/{task_id}/callback"
    }

    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode('utf-8'),
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {token}"
        }
    )

    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode('utf-8'))
            print(f"✅ [HTTP {resp.status} ACCEPTED] Despacho enviado com sucesso!")
            print(json.dumps(data, indent=2, ensure_ascii=False))
            return True, data
    except urllib.error.HTTPError as e:
        body = e.read().decode('utf-8', errors='ignore')
        print(f"❌ [HTTP {e.code} ERROR] Falha no despacho: {body}")
        return False, body
    except Exception as e:
        print(f"❌ Erro de conexão: {e}")
        return False, str(e)

def listen_websocket(ws_url):
    try:
        import websockets
        import asyncio
    except ImportError:
        print("Biblioteca 'websockets' necessária para o modo listen.")
        print("Instale com: pip install websockets")
        return

    async def _listener():
        print(f"📡 Conectando ao WebSocket: {ws_url}...")
        async with websockets.connect(ws_url) as ws:
            print("🟢 Conectado ao hub do The Grid Metropolis! Aguardando eventos...\n")
            while True:
                msg = await ws.recv()
                data = json.loads(msg)
                topic = data.get("topic", "UNKNOWN")
                print(f"🔔 [{topic}] Recebido:")
                print(json.dumps(data, indent=2, ensure_ascii=False))
                print("-" * 50)

    try:
        asyncio.run(_listener())
    except KeyboardInterrupt:
        print("\nDesconectado.")

def run_e2e_simulation(http_url, ws_url, token):
    try:
        import websockets
        import asyncio
    except ImportError:
        print("Bibliotecas 'websockets' necessárias para o teste e2e.")
        return False

    async def _test():
        print(f"1. Conectando ao WebSocket {ws_url}...")
        async with websockets.connect(ws_url) as ws:
            first_msg = await ws.recv()
            snapshot = json.loads(first_msg)
            print("2. Snapshot recebido com sucesso! Tópico:", snapshot.get("topic"))
            assert snapshot.get("topic") == "WORLD_SNAPSHOT", "Primeiro evento deve ser WORLD_SNAPSHOT"

            test_task_id = f"task-mock-{int(time.time())}"
            print(f"3. Enviando despacho HTTP para {test_task_id}...")
            ok, _ = send_dispatch(
                http_url=http_url,
                token=token,
                task_id=test_task_id,
                title="Mock E2E Task",
                agent_id="agent-rust-dev",
                project_id="proj-notex-backend",
                priority="HIGH",
                prompt="Validando fluxo end-to-end..."
            )
            assert ok, "Falha na requisição HTTP de despacho"

            print("4. Aguardando evento TASK_ASSIGNED no WebSocket...")
            event_msg = await ws.recv()
            event = json.loads(event_msg)
            print("5. Evento recebido no WS:", event.get("topic"))
            assert event.get("topic") == "TASK_ASSIGNED", "Evento deve ser TASK_ASSIGNED"
            assert event.get("payload", {}).get("taskId") == test_task_id
            assert event.get("payload", {}).get("agentId") == "agent-rust-dev"
            print("\n🎉 TESTE DE INTEGRAÇÃO END-TO-END BEM-SUCEDIDO!")
            return True

    return asyncio.run(_test())

def main():
    parser = argparse.ArgumentParser(description="CLI Mock & Testes de Integração da Ponte The Grid Metropolis")
    parser.add_argument("--dispatch", action="store_true", help="Envia um despacho de tarefa mock via HTTP")
    parser.add_argument("--listen", action="store_true", help="Escuta eventos do WebSocket em tempo real")
    parser.add_argument("--e2e", action="store_true", help="Executa o teste end-to-end completo (HTTP + WS)")
    parser.add_argument("--task-id", default=f"task-cli-{int(time.time())}", help="ID da tarefa")
    parser.add_argument("--title", default="Implementar novo endpoint", help="Título da tarefa")
    parser.add_argument("--agent-id", default="agent-rust-dev", help="ID do agente de destino")
    parser.add_argument("--project-id", default="proj-notex-backend", help="ID do Pod do projeto")
    parser.add_argument("--priority", default="HIGH", choices=["LOW", "MEDIUM", "HIGH", "CRITICAL"])
    parser.add_argument("--prompt", default="Construir rotas de integração...", help="Prompt da tarefa")
    parser.add_argument("--url", default=DEFAULT_HTTP_URL, help="URL base HTTP do servidor")
    parser.add_argument("--ws-url", default=DEFAULT_WS_URL, help="URL do endpoint WebSocket")
    parser.add_argument("--token", default=DEFAULT_TOKEN, help="Bearer token de autenticação")

    args = parser.parse_args()

    if args.listen:
        listen_websocket(args.ws_url)
    elif args.e2e:
        run_e2e_simulation(args.url, args.ws_url, args.token)
    elif args.dispatch:
        send_dispatch(
            http_url=args.url,
            token=args.token,
            task_id=args.task_id,
            title=args.title,
            agent_id=args.agent_id,
            project_id=args.project_id,
            priority=args.priority,
            prompt=args.prompt
        )
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
