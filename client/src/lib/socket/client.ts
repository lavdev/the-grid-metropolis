import type { WebSocketEvent } from '../../types';

export type ConnectionState = 'CONNECTING' | 'CONNECTED' | 'DISCONNECTED';

type EventListener = (event: WebSocketEvent) => void;
const MAX_BUFFERED_EVENTS = 100;

export class GridWebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private listeners: Set<EventListener> = new Set();
  private stateListeners: Set<(state: ConnectionState) => void> = new Set();
  private reconnectTimeout: number | null = null;
  private reconnectAttempts = 0;
  private maxReconnectDelay = 10000;
  private intentionalDisconnect = false;
  private bufferedEvents: WebSocketEvent[] = [];
  public connectionState: ConnectionState = 'DISCONNECTED';

  constructor(url?: string) {
    if (url) {
      this.url = url;
    } else {
      const isTauriDesktop = window.location.protocol === 'tauri:' || window.location.hostname === 'tauri.localhost';
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      this.url = isTauriDesktop ? 'ws://127.0.0.1:3000/ws' : `${protocol}//${window.location.host}/ws`;
    }
  }

  public connect() {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.intentionalDisconnect = false;
    if (this.reconnectTimeout !== null) {
      window.clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    this.setConnectionState('CONNECTING');
    try {
      const socket = new WebSocket(this.url);
      this.ws = socket;

      socket.onopen = () => {
        if (this.ws !== socket) return;
        this.reconnectAttempts = 0;
        this.setConnectionState('CONNECTED');
        console.log('⚡ Conectado ao hub The Grid Metropolis via WebSocket');
      };

      socket.onmessage = (event) => {
        if (this.ws !== socket) return;
        try {
          const parsed: WebSocketEvent = JSON.parse(event.data);
          this.notifyListeners(parsed);
        } catch (err) {
          console.error('Falha ao decodificar evento WebSocket:', err);
        }
      };

      socket.onclose = () => {
        if (this.ws !== socket) return;
        this.ws = null;
        this.setConnectionState('DISCONNECTED');
        if (!this.intentionalDisconnect) this.scheduleReconnect();
      };

      socket.onerror = (err) => {
        if (this.ws !== socket) return;
        console.warn('Alerta na conexão WebSocket:', err);
      };
    } catch (e) {
      this.setConnectionState('DISCONNECTED');
      this.scheduleReconnect();
    }
  }

  public subscribe(listener: EventListener): () => void {
    this.listeners.add(listener);
    if (this.bufferedEvents.length > 0) {
      const pendingEvents = this.bufferedEvents;
      this.bufferedEvents = [];
      pendingEvents.forEach(listener);
    }
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
    if (this.listeners.size === 0) {
      this.bufferedEvents.push(event);
      if (this.bufferedEvents.length > MAX_BUFFERED_EVENTS) {
        this.bufferedEvents.shift();
      }
      return;
    }

    this.listeners.forEach((listener) => listener(event));
  }

  private scheduleReconnect() {
    if (this.intentionalDisconnect || this.reconnectTimeout !== null) return;

    const delay = Math.min(1000 * Math.pow(1.5, this.reconnectAttempts), this.maxReconnectDelay);
    this.reconnectAttempts++;

    this.reconnectTimeout = window.setTimeout(() => {
      this.reconnectTimeout = null;
      console.log(`Tentando reconectar ao WebSocket (${this.reconnectAttempts})...`);
      this.connect();
    }, delay);
  }

  public disconnect() {
    this.intentionalDisconnect = true;
    if (this.reconnectTimeout !== null) {
      window.clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
    const socket = this.ws;
    this.ws = null;
    if (socket) {
      socket.close();
    }
    this.setConnectionState('DISCONNECTED');
  }
}
