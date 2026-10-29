with open('client/src/lib/stores/projectStore.ts', 'r', encoding='utf-8') as f:
    content = f.read()

start_marker = 'const INITIAL_PROJECTS: ProjectTerminal[] = ['
end_marker = "export const projectSearchQuery = writable<string>('');"

start_idx = content.find(start_marker)
end_idx = content.find(end_marker) + len(end_marker)

new_block = """const INITIAL_PROJECTS: ProjectTerminal[] = [
  {
    id: 'proj-notex',
    name: 'The Notex',
    path: 'C:\\\\Workspace\\\\notex',
    model: 'CODEX CLI (ASSINATURA)',
    status: 'ACTIVE',
    accentColor: '#00f0ff',
    metrics: {
      tokensIn: 0,
      tokensOut: 0,
      latencyMs: 0,
      cpu: '||      ',
      mem: '|       ',
    },
    activeDroidId: 'spark-01',
    pipeline: {
      isActive: false,
      stage: 'IDLE',
      currentDroidId: 'spark-01',
      prompt: '',
      progressPercent: 0,
      statusText: 'Aguardando instrução do operador para The Notex',
    },
    droids: [
      {
        id: 'spark-01',
        name: 'SPARK-01',
        role: 'ARQUITETO',
        model: 'Codex Lead',
        planBadge: 'Lead Architect',
        avatarType: 'spark',
        status: 'ONLINE',
        statusDetail: 'Pronto para planejamento de arquitetura e requisitos',
        tokens: 0,
        accentColor: '#00f0ff',
      },
      {
        id: 'agent-codex-01',
        name: 'Rusty',
        role: 'PROGRAMADOR',
        model: 'Codex Systems',
        planBadge: 'Systems Coder',
        avatarType: 'hacker',
        status: 'ONLINE',
        statusDetail: 'Pronto para codificação técnica em C:\\\\Workspace\\\\notex',
        tokens: 0,
        accentColor: '#39ff14',
      },
      {
        id: 'agent-sec-05',
        name: 'Vanguard',
        role: 'REVISOR_DE_CODIGO',
        model: 'Codex QA',
        planBadge: 'Code Reviewer & QA',
        avatarType: 'sentinel',
        status: 'ONLINE',
        statusDetail: 'Pronto para revisão de código, AST e testes',
        tokens: 0,
        accentColor: '#ff007f',
      },
    ],
    messages: [
      {
        id: 'msg-notex-sys',
        sender: 'system',
        content: 'Workspace "The Notex" conectado em C:\\\\Workspace\\\\notex. Tríade oficial de droids (SPARK-01, Rusty, Vanguard) pronta.',
        timestamp: '16:58',
      },
      {
        id: 'msg-notex-spark',
        sender: 'agent',
        agentName: 'SPARK-01',
        agentRole: 'ARQUITETO',
        content: 'Tríade a postos para The Notex, Arquiteto! Aguardando sua próxima instrução para iniciarmos a esteira técnica.',
        timestamp: '16:58',
        status: 'DONE',
      },
    ],
  },
];

export const projects = writable<ProjectTerminal[]>(INITIAL_PROJECTS);
export const activeProjectId = writable<string>('proj-notex');
export const isSidebarCollapsed = writable<boolean>(false);
export const projectSearchQuery = writable<string>('');"""

if start_idx != -1 and end_idx != -1:
    content = content[:start_idx] + new_block + content[end_idx:]
    
    # Also inject workingDir: currentProj.path into the streamAiCompletion calls
    content = content.replace(
        "prompt: commandText,\n        droidName: soloDroid.name,",
        "prompt: commandText,\n        workingDir: currentProj.path,\n        droidName: soloDroid.name,"
    )
    content = content.replace(
        "prompt: `Instrução do usuário: \"${commandText}\"",
        "workingDir: currentProj.path,\n      prompt: `Instrução do usuário: \"${commandText}\""
    )
    content = content.replace(
        "prompt: `Com base na especificação do Arquiteto:",
        "workingDir: currentProj.path,\n      prompt: `Com base na especificação do Arquiteto:"
    )
    content = content.replace(
        "prompt: `Audite o código implementado:",
        "workingDir: currentProj.path,\n      prompt: `Audite o código implementado:"
    )

    with open('client/src/lib/stores/projectStore.ts', 'w', encoding='utf-8') as f:
        f.write(content)
    print('SUCCESS')
else:
    print('FAILED', start_idx, end_idx)
