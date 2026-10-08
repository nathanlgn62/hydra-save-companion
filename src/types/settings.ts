export interface UserSettings {
  uploadInterval: "afterGameClose" | "manually";
  downloadInterval: number | "manually";
  autoStartWithSystem: boolean;
  desktopNotificationsEnabled?: boolean;
  demoMode?: boolean;
}
