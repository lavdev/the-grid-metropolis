import type { WebSocketEvent } from '../../types';

export type ConnectionState = 'CONNECTING' | 'CONNECTED' | 'DISCONNECTED';

type EventListener = (event: WebSocketEvent) => void;

export class GridWebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private listeners: Set<EventListener> = new Set();
  private stateListeners: Set<(state: ConnectionState) => void> = new Set();
  private reconnectTimeout: number | null = null;
  private reconnectAttempts = 0;
  private maxReconnectDelay = 10000;
  public connectionState: ConnectionState = 'DISCONNECTED';

  constructor(url?: string) {
    if (url) {
      this.url = url;
    } else {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      this.url = `${protocol}//${window.location.host}/ws`;
    }
  }

  public connect() {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.setConnectionState('CONNECTING');
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.setConnectionState('CONNECTED');
        console.log('⚡ Conectado ao hub The Grid Metropolis via WebSocket');
      };

      this.ws.onmessage = (event) => {
        try {
          const parsed: WebSocketEvent = JSON.parse(event.data);
          this.notifyListeners(parsed);
        } catch (err) {
          console.error('Falha ao decodificar evento WebSocket:', err);
        }
      };

      this.ws.onclose = () => {
        this.setConnectionState('DISCONNECTED');
        this.scheduleReconnect();
      };

      this.ws.onerror = (err) => {
        console.warn('Alerta na conexão WebSocket:', err);
      };
    } catch (e) {
      this.setConnectionState('DISCONNECTED');
      this.scheduleReconnect();
    }
  }

  public subscribe(listener: EventListener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  public onStateChange(listener: (state: ConnectionState) => void): () => void {
    this.stateListeners.add(listener);
    listener(this.connectionState);
    return () => this.stateListeners.delete(listener);
  }

  private setConnectionState(state: ConnectionState) {
    this.connectionState = state;
    this.stateListeners.forEach((listener) => listener(state));
  }

  private notifyListeners(event: WebSocketEvent) {
    this.listeners.forEach((listener) => listener(event));
  }

  private scheduleReconnect() {
    if (this.reconnectTimeout) return;

    const delay = Math.min(1000 * Math.pow(1.5, this.reconnectAttempts), this.maxReconnectDelay);
    this.reconnectAttempts++;

    this.reconnectTimeout = window.setTimeout(() => {
      this.reconnectTimeout = null;
      console.log(`Tentando reconectar ao WebSocket (${this.reconnectAttempts})...`);
      this.connect();
    }, delay);
  }

  public disconnect() {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.setConnectionState('DISCONNECTED');
  }
}
