export interface SpriteConfig {
  textureKey: string;
  skinTheme: 'cyber_punk_lab' | 'street_hacker' | 'android_unit';
  colorAccent: string;
}

export interface AgentLocation {
  x: number;
  y: number;
  currentZone: 'NEXUS' | 'HOME' | 'LOUNGE' | 'GLITCH_ALLEY' | 'TRANSIT';
}

export interface AgentMetrics {
  tasksCompleted: number;
  tokensConsumed: number;
  uptimeSeconds: number;
}

export type AgentRole = 'DEVELOPER' | 'RESEARCHER' | 'DEVOPS' | 'SECURITY' | 'BOT_RUNNER';
export type AgentStatus = 'IDLE' | 'WALKING' | 'WORKING' | 'MEETING' | 'ERROR';

export interface Agent {
  id: string;
  name: string;
  role: AgentRole;
  spriteConfig: SpriteConfig;
  homePodId: string;
  currentLocation: AgentLocation;
  status: AgentStatus;
  activeTaskId?: string;
  metrics: AgentMetrics;
}

export interface VisualState {
  powerLevel: number;
  chimneySmokeColor: string;
  screenStatus: 'OFF' | 'CODE_STREAM' | 'DASHBOARD' | 'ALERT_RED';
}

export interface InteriorStation {
  stationId: string;
  isOccupied: boolean;
  coordX: number;
  coordY: number;
}

export interface Pod {
  id: string;
  title: string;
  repositoryUrl?: string;
  maxCapacity: number;
  assignedAgents: string[];
  visualState: VisualState;
  interiorStations: InteriorStation[];
}

export type TaskPriority = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
export type TaskStatus = 'PENDING' | 'IN_PROGRESS' | 'REVIEW' | 'COMPLETED' | 'FAILED';

export interface TaskPayload {
  prompt: string;
  contextFiles?: string[];
  scratchpad?: string;
}

export interface Task {
  taskId: string;
  projectId: string;
  targetAgentId: string;
  title: string;
  priority: TaskPriority;
  status: TaskStatus;
  payload: TaskPayload;
  createdAt: string;
  completedAt?: string;
}

export type EventTopic =
  | 'WORLD_SNAPSHOT'
  | 'AGENT_SPAWN'
  | 'TASK_ASSIGNED'
  | 'TASK_PROGRESS'
  | 'AGENT_COLLABORATE'
  | 'COLLABORATION_REQUEST'
  | 'TASK_COMPLETED'
  | 'TASK_ERROR'
  | 'AGENT_REST';

export interface WebSocketEvent<T = any> {
  topic: EventTopic;
  timestamp: number;
  payload: T;
}
