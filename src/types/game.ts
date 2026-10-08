export interface RemoteBackupInfo {
  file_id: string;
  name: string;
  modified_time: string;
}

export interface HydraGame {
  title: string;
  objectId: string;
  shop?: string;
  executablePath?: string;
  iconUrl?: string;
  cover?: string;
  lastLocalSave?: string;
  lastRemoteSave?: string;
  savePath: string;
  pathExists: boolean;
  ludasaviPathExists: boolean;
  localPathExists: boolean;
  backups?: RemoteBackupInfo[];
}
