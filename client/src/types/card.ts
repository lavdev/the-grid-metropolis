export type CardRarity = 'COMMON' | 'UNCOMMON' | 'RARE' | 'LEGENDARY';

export type CardCategory = 'CODE' | 'ARCH' | 'TEST' | 'RESEARCH' | 'SECURITY' | 'CUSTOM';

export type TargetAIModel =
  | 'Claude 3.7 Sonnet'
  | 'Codex / GPT-4.5'
  | 'Gemini 2.0 Flash'
  | 'DeepSeek R1';

export interface ActionCard {
  id: string;
  title: string;
  subtitle: string;
  category: CardCategory;
  rarity: CardRarity;
  energyCost: number;       // Custo em energia (1 a 5)
  tokenEstimate: string;    // ex: "2.4k tokens"
  targetModel: TargetAIModel;
  targetRole: 'DEVELOPER' | 'RESEARCHER' | 'DEVOPS' | 'SECURITY' | 'BOT_RUNNER';
  description: string;
  prompt: string;
  contextFiles?: string[];
  flavorText: string;
  tags: string[];
}

export interface PlayedCardRecord {
  recordId: string;
  card: ActionCard;
  taskId: string;
  assignedAgentId: string;
  assignedAgentName: string;
  playedAt: string;
  completedAt?: string;
  status: 'PENDING' | 'EXECUTING' | 'RESOLVED' | 'FAILED';
  tokensUsed?: number;
  reasoningSnippet?: string;
  artifactsCreated?: string[];
}
