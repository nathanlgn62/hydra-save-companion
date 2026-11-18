import { invoke } from "@tauri-apps/api/core";
import { CloudAuthTokenResult } from "../types/cloud";

export async function getReportAuthToken(): Promise<{
  token: string;
  provider: "google-drive" | "dropbox";
}> {
  const token =
    localStorage.getItem("cloud_token") || localStorage.getItem("gdrive_token");
  const rawProvider =
    localStorage.getItem("cloud_provider") ||
    (localStorage.getItem("gdrive_token") ? "google-drive" : null);

  const provider = rawProvider === "dropbox" ? "dropbox" : "google-drive";

  if (
    !token ||
    (rawProvider !== "google-drive" &&
      rawProvider !== "dropbox" &&
      rawProvider !== "google")
  ) {
    throw new Error(
      "You must be signed in with a Google or Dropbox account to perform this action.",
    );
  }

  const result = await invoke<CloudAuthTokenResult>("get_cloud_auth_token", {
    token,
    provider,
  });

  if (result.updatedCloudToken) {
    localStorage.setItem("cloud_token", result.updatedCloudToken);
  }

  return {
    token: result.token,
    provider: (result.provider === "dropbox"
      ? "dropbox"
      : "google-drive") as "google-drive" | "dropbox",
  };
}
