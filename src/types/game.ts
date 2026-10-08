export interface RemoteBackupInfo {
  file_id: string;
  name: string;
  modified_time: string;
}

export type SaveStatusFilter =
  | "all"
  | "up-to-date"
  | "local-newer"
  | "cloud-newer"
  | "to-download"
  | "never-synced"
  | "no-save";

export interface HydraGame {
  title: string;
  objectId: string;
  shop?: string;
  executablePath?: string;
  iconUrl?: string;
  cover?: string;
  lastLocalSave?: string;
  lastRemoteSave?: string;
  savePath: string | null;
  pathExists?: boolean;
  ludasaviPathExists?: boolean;
  localPathExists?: boolean;
  backups?: RemoteBackupInfo[];
}

export interface GameProcessInfo {
  title: string;
  executable_name: string;
  save_path: string | null;
}

export interface GameClosedPayload {
  title: string;
  savePath?: string;
}
