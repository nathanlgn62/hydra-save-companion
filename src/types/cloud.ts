export type CloudProviderId =
  | "google-drive"
  | "dropbox"
  | "proton-drive"
  | "mega";

export interface CloudStatus {
  isConnected: boolean;
  provider: string | null;
}

export interface CloudStorageInfo {
  provider: string;
  providerName: string;
  usedBytes: number;
  totalBytes?: number | null;
  usedFormatted: string;
  totalFormatted?: string | null;
  freeBytes?: number | null;
  freeFormatted?: string | null;
}
