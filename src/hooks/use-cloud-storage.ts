import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useCloudStatus } from "../stores/cloud-store";
import { CloudStorageInfo } from "../types/cloud";

export function useCloudStorage() {
  const { isDriveConnected, cloudProvider } = useCloudStatus();

  return useQuery<CloudStorageInfo | null>({
    queryKey: ["cloud-storage", cloudProvider],
    queryFn: async () => {
      const token =
        localStorage.getItem("cloud_token") ||
        localStorage.getItem("gdrive_token") ||
        "";

      if (!isDriveConnected || !token) {
        return null;
      }

      try {
        return await invoke<CloudStorageInfo>("get_cloud_storage_info", {
          token,
          provider: cloudProvider || "google-drive",
        });
      } catch (err) {
        console.warn(
          "Impossible de récupérer les informations de quota Cloud :",
          err,
        );
        return null;
      }
    },
    enabled: isDriveConnected,
    refetchInterval: 1000 * 60,
    staleTime: 1000 * 5,
  });
}
