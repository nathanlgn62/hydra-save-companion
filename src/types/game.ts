export interface HydraGame {
  title: string;
  objectId: string;
  shop?: string;
  executablePath?: string;
  iconUrl?: string;
  lastLocalSave?: string;
  lastRemoteSave?: string;
  savePath: string;
}
