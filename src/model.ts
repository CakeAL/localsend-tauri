export interface DeviceMessage {
  alias: string;
  deviceModel: string;
  deviceType: string;
  download: boolean;
  fingerprint: string;
  port: number;
  protocol: string;
  version: string;
}

export interface FileRequest {
  info: DeviceMessage;
  files: Record<string, FileInfo>;
}

export interface FileInfo {
  id: string;
  fileName: string;
  size: number; // bytes
  fileType: string;
  sha256?: string; // Optional
  preview?: Uint8Array; // Optional
  downloaded?: number;
  speed?: number;
  progress?: number;
}
