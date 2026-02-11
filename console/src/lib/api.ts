const API_BASE_URL = import.meta.env.VITE_API_URL || "/api";

export interface Device {
  id: number;
  mac_address: string;
  ip_address: string | null;
  hostname: string | null;
  nickname: string | null;
  status: "online" | "offline" | "unknown";
  first_seen: string;
  last_seen: string;
}

export interface DevicesResponse {
  devices: Device[];
}

export interface Rule {
  id: number;
  name: string;
  description: string | null;
  trigger_type:
    | "new_device"
    | "device_connected"
    | "device_disconnected"
    | "device_status_change";
  mac_filter: string | null;
  enabled: boolean;
  notification_channels: string[];
  created_at: string;
  updated_at: string;
}

export interface RulesResponse {
  rules: Rule[];
}

export interface CreateRuleRequest {
  name: string;
  description: string | null;
  trigger_type: Rule["trigger_type"];
  mac_filter: string | null;
  enabled: boolean;
  notification_channels: string[];
}

export interface Metrics {
  total_devices: number;
  online_devices: number;
  offline_devices: number;
  total_rules: number;
  enabled_rules: number;
  packets_captured: number;
  notifications_sent: number;
  uptime_seconds: number;
}

export interface TelegramChannel {
  type: "telegram";
  bot_token: string;
  chat_id: string;
}

export interface NtfyChannel {
  type: "ntfy";
  server_url: string;
  topic: string;
  token?: string;
}

export interface WebhookChannel {
  type: "webhook";
  url: string;
  headers?: Record<string, string>;
}

export type NotificationChannel =
  | TelegramChannel
  | NtfyChannel
  | WebhookChannel;

export interface Config {
  daemon: {
    interface: string;
    neighbor_check_interval_secs: number;
    device_timeout_secs: number;
    capture_filter: string | null;
    log_cleanup_enabled: boolean;
    log_retention_days: number;
  };
  api: {
    host: string;
    port: number;
  };
  database: {
    path: string;
  };
  notifications: NotificationChannel[];
}

export interface HealthResponse {
  status: string;
  service: string;
  uptime_seconds: number;
  system: {
    cpu_usage_percent: number;
    memory_usage_percent: number;
    total_memory_mb: number;
    used_memory_mb: number;
  };
}

export interface LogEntry {
  id: number;
  timestamp: string;
  level: "info" | "warning" | "error" | "debug";
  category: string;
  message: string;
  details: string | null;
}

export interface LogsResponse {
  logs: LogEntry[];
  count: number;
}

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options?: RequestInit,
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...options?.headers,
      },
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`API Error: ${response.status} - ${errorText}`);
    }

    return response.json();
  }

  async getHealth(): Promise<HealthResponse> {
    return this.request<HealthResponse>("/health");
  }

  async getDevices(): Promise<DevicesResponse> {
    return this.request<DevicesResponse>("/devices");
  }

  async getDevice(macAddress: string): Promise<Device> {
    return this.request<Device>(`/devices/${encodeURIComponent(macAddress)}`);
  }

  async updateDeviceNickname(
    macAddress: string,
    nickname: string | null,
  ): Promise<Device> {
    return this.request<Device>(
      `/devices/${encodeURIComponent(macAddress)}/nickname`,
      {
        method: "POST",
        body: JSON.stringify({ nickname }),
      },
    );
  }

  async getRules(): Promise<RulesResponse> {
    return this.request<RulesResponse>("/rules");
  }

  async createRule(rule: CreateRuleRequest): Promise<Rule> {
    return this.request<Rule>("/rules", {
      method: "POST",
      body: JSON.stringify(rule),
    });
  }

  async updateRule(
    id: number,
    rule: Partial<CreateRuleRequest>,
  ): Promise<Rule> {
    return this.request<Rule>(`/rules/${id}`, {
      method: "PUT",
      body: JSON.stringify(rule),
    });
  }

  async deleteRule(id: number): Promise<void> {
    await this.request<void>(`/rules/${id}`, {
      method: "DELETE",
    });
  }

  async getConfig(): Promise<Config> {
    return this.request<Config>("/config");
  }

  async updateConfig(config: Partial<Config>): Promise<Config> {
    return this.request<Config>("/config", {
      method: "POST",
      body: JSON.stringify(config),
    });
  }

  async getMetrics(): Promise<Metrics> {
    return this.request<Metrics>("/metrics");
  }

  async restartDaemon(): Promise<void> {
    await this.request<{ message: string }>("/restart", {
      method: "POST",
    });
  }

  async getLogs(): Promise<LogsResponse> {
    return this.request<LogsResponse>("/logs");
  }
}

export const api = new ApiClient(API_BASE_URL);

export function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (days > 0) {
    return `${days}d ${hours}h`;
  } else if (hours > 0) {
    return `${hours}h ${minutes}m`;
  } else {
    return `${minutes}m`;
  }
}

export function getStatusColor(status: Device["status"]): string {
  switch (status) {
    case "online":
      return "success";
    case "offline":
      return "danger";
    default:
      return "secondary";
  }
}

export function getTriggerTypeLabel(type: Rule["trigger_type"]): string {
  switch (type) {
    case "new_device":
      return "New Device";
    case "device_connected":
      return "Device Connected";
    case "device_disconnected":
      return "Device Disconnected";
    case "device_status_change":
      return "Status Change";
    default:
      return type;
  }
}
