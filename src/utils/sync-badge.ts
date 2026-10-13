import {
  AlertCircle,
  CheckCircle2,
  Cloud,
  CloudDownload,
  CloudOff,
  CloudUpload,
} from "lucide-react";
import { SaveStatusFilter, SyncBadgeConfig, SyncBadgeVariant } from "../types";
import { parseSaveDate } from "./date";

export type { SaveStatusFilter, SyncBadgeConfig, SyncBadgeVariant };

export function getSyncBadgeConfig(
  ludasaviPathExists?: boolean,
  lastLocalSave?: string | null,
  lastRemoteSave?: string | null,
  isCloudConnected: boolean = true,
): SyncBadgeConfig {
  if (!ludasaviPathExists) {
    return {
      label: "Not Found",
      variant: "neutral",
      Icon: AlertCircle,
      description:
        "No save directory was found via the Ludasavi manifest.",
    };
  }

  const localDate = parseSaveDate(lastLocalSave);

  // If no cloud service is connected
  if (!isCloudConnected) {
    if (!localDate) {
      return {
        label: "No Save",
        variant: "neutral",
        Icon: Cloud,
        description: "No local save files detected.",
      };
    }

    return {
      label: "Not Synced",
      variant: "neutral",
      Icon: CloudOff,
      description:
        "Local save found. Connect a cloud service to synchronize.",
    };
  }

  const remoteDate = parseSaveDate(lastRemoteSave);

  // If neither local nor cloud has a save
  if (!localDate && !remoteDate) {
    return {
      label: "No Save",
      variant: "neutral",
      Icon: Cloud,
      description:
        "No save files detected locally or in the cloud.",
    };
  }

  // If we have a cloud save but no local save
  if (!localDate && remoteDate) {
    return {
      label: "To Download",
      variant: "info",
      Icon: CloudDownload,
      description:
        "A backup is available in the cloud, but no local files exist.",
    };
  }

  if (!localDate) {
    return {
      label: "No Save",
      variant: "neutral",
      Icon: Cloud,
      description: "No local save files detected.",
    };
  }

  if (!remoteDate) {
    return {
      label: "Never Synced",
      variant: "warning",
      Icon: Cloud,
      description:
        "Local save exists but has never been uploaded to the cloud.",
    };
  }

  const timeDiff = localDate - remoteDate;

  if (Math.abs(timeDiff) < 60000) {
    return {
      label: "Up to Date",
      variant: "success",
      Icon: CheckCircle2,
      description: "Your local and cloud saves are in sync.",
    };
  }

  if (localDate > remoteDate) {
    return {
      label: "Local Newer",
      variant: "info",
      Icon: CloudUpload,
      description:
        "Your local save is newer than the cloud version.",
    };
  }

  return {
    label: "Cloud Newer",
    variant: "warning",
    Icon: CloudDownload,
    description:
      "The cloud version is newer than your local save.",
  };
}

export function getGameStatusCategory(
  ludasaviPathExists?: boolean,
  lastLocalSave?: string | null,
  lastRemoteSave?: string | null,
  isCloudConnected: boolean = true,
): SaveStatusFilter {
  const badge = getSyncBadgeConfig(
    ludasaviPathExists,
    lastLocalSave,
    lastRemoteSave,
    isCloudConnected,
  );

  switch (badge.label) {
    case "Up to Date":
    case "À jour":
      return "up-to-date";
    case "Local Newer":
    case "Local plus récent":
      return "local-newer";
    case "Cloud Newer":
    case "Cloud plus récent":
      return "cloud-newer";
    case "To Download":
    case "À télécharger":
      return "to-download";
    case "Not Synced":
    case "Non synchronisé":
    case "Never Synced":
    case "Jamais sync":
      return "never-synced";
    case "No Save":
    case "Aucune sauvegarde":
    case "Not Found":
    case "Introuvable":
    default:
      return "no-save";
  }
}
