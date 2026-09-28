export interface HydraGame {
  title: string;
  objectId: string;
  shop?: string;
  executablePath?: string;
  isDeleted?: boolean;
  iconUrl?: string;
  lastLocalSave: string;
  savePath: string;
}
