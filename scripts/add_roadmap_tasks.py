import sys
sys.path.append('scripts')
from notion_sync import HEADERS, DATABASE_ID
import urllib.request
import json

tasks_to_create = [
    {
        'id': 'T-6.13',
        'title': 'Efeitos Sonoros Retrô 8-bit (Web Audio API Synthesizer)',
        'phase': 'Fase 6: HUD & Modo Diretor',
        'priority': 'P2',
        'deps': 'T-6.8, T-6.12',
        'desc': 'Sintetizador de áudio Web Audio API integrado sem assets externos pesados: cliques mecânicos de teclado na digitação CLI, zumbido e bips de decolagem do SPARK-01 e fanfarra triunfante em 8-bit ao concluir tarefas.'
    },
    {
        'id': 'T-6.14',
        'title': 'Integração Real com Modelos de IA e Streaming no Terminal CRT',
        'phase': 'Fase 6: HUD & Modo Diretor',
        'priority': 'P1',
        'deps': 'T-6.9, T-1.4',
        'desc': 'Conectar a linha de comando do terminal às APIs reais de LLMs (Gemini, Claude, OpenAI) com suporte a streaming de tokens em tempo real, exibição de Chain-of-Thought e execução de ferramentas nos arquivos locais.'
    },
    {
        'id': 'T-6.15',
        'title': 'Leitor e Inspetor de Código Retrô no Monitor CRT (Estilo Vim/Nano)',
        'phase': 'Fase 6: HUD & Modo Diretor',
        'priority': 'P2',
        'deps': 'T-6.9, T-6.8',
        'desc': 'Aba ou modo de visualização no monitor CRT para inspecionar e navegar pelo código-fonte dos arquivos dos workspaces (ex: Rust, TypeScript, SQL), com syntax highlighting em tons de fósforo verde/âmbar e atalhos de teclado.'
    },
    {
        'id': 'T-6.16',
        'title': 'Gerenciador e Criador de Droids Personalizados por Workspace',
        'phase': 'Fase 6: HUD & Modo Diretor',
        'priority': 'P2',
        'deps': 'T-6.9',
        'desc': 'Painel e modal para cadastrar e vincular novos droids a qualquer projeto, escolhendo modelo de IA associado, papel especializado (Dev, QA, Security), cor de acento néon e avatar pixel art.'
    }
]

url = 'https://api.notion.com/v1/pages'

for t in tasks_to_create:
    payload = {
        'parent': {'database_id': DATABASE_ID},
        'properties': {
            'ID': {'rich_text': [{'text': {'content': t['id']}}]},
            'Tarefa': {'title': [{'text': {'content': t['title']}}]},
            'Status': {'status': {'name': 'Not started'}},
            'Fase': {'select': {'name': t['phase']}},
            'Prioridade': {'select': {'name': t['priority']}},
            'Dependências': {'rich_text': [{'text': {'content': t['deps']}}]}
        },
        'children': [
            {
                'object': 'block',
                'type': 'paragraph',
                'paragraph': {
                    'rich_text': [{'type': 'text', 'text': {'content': t['desc']}}]
                }
            }
        ]
    }
    req = urllib.request.Request(url, headers=HEADERS, data=json.dumps(payload).encode('utf-8'))
    with urllib.request.urlopen(req) as resp:
        res = json.loads(resp.read().decode())
        print(f"Created {t['id']} in Notion (Page ID: {res['id']})")
