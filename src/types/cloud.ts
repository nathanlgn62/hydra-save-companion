export type CloudProviderId =
  | "google-drive"
  | "dropbox"
  | "proton-drive"
  | "mega";

export interface CloudStatus {
  isConnected: boolean;
  provider: string | null;
}
