import { LucideIcon } from "lucide-react";
import { RemoteBackupInfo } from "./game";

export type SyncStatusType =
  | "UpToDate"
  | "LocalNewer"
  | "CloudNewer"
  | "CloudOnly"
  | "NotFound";

export interface SyncStatusResult {
  status: string;
  localTime: string;
  cloudTime: string;
  backups: RemoteBackupInfo[];
}

export interface SaveInfoResponse {
  ludasaviPathExists: boolean;
  localPathExists: boolean;
  resolvedPath: string | null;
  lastModified: string | null;
}

export interface SyncPayload {
  gameTitle: string;
  savePath: string;
  fileId?: string;
}

export type SyncBadgeVariant =
  | "neutral"
  | "warning"
  | "success"
  | "info"
  | "danger";

export interface SyncBadgeConfig {
  label: string;
  variant: SyncBadgeVariant;
  Icon: LucideIcon;
  description?: string;
}
