import {
  AlertCircle,
  CheckCircle2,
  Cloud,
  CloudDownload,
  CloudUpload,
  LucideIcon,
} from "lucide-react";

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

function parseSaveDate(dateStr?: string | null): Date | null {
  if (!dateStr || dateStr === "Jamais") return null;

  const customFormatRegex = /^(\d{2})\/(\d{2})\/(\d{4})\s+(\d{2}):(\d{2})$/;
  const match = dateStr.match(customFormatRegex);

  if (match) {
    const [, day, month, year, hours, minutes] = match;
    return new Date(
      Number(year),
      Number(month) - 1,
      Number(day),
      Number(hours),
      Number(minutes),
    );
  }

  const parsed = new Date(dateStr);
  return isNaN(parsed.getTime()) ? null : parsed;
}

export function getSyncBadgeConfig(
  ludasaviPathExists?: boolean,
  lastLocalSave?: string | null,
  lastRemoteSave?: string | null,
): SyncBadgeConfig {
  if (!ludasaviPathExists) {
    return {
      label: "Introuvable",
      variant: "neutral",
      Icon: AlertCircle,
      description:
        "Aucun dossier de sauvegarde n'a été trouvé via le manifeste Ludasavi.",
    };
  }

  const localDate = parseSaveDate(lastLocalSave);
  const remoteDate = parseSaveDate(lastRemoteSave);

  // Si ni le local ni le cloud n'ont de sauvegarde
  if (!localDate && !remoteDate) {
    return {
      label: "Aucune sauvegarde",
      variant: "neutral",
      Icon: Cloud,
      description:
        "Aucune sauvegarde n'a été détectée ni en local ni sur le cloud.",
    };
  }

  // Si on a une save sur le cloud mais pas de save locale
  if (!localDate && remoteDate) {
    return {
      label: "À télécharger",
      variant: "info",
      Icon: CloudDownload,
      description:
        "Une sauvegarde est disponible sur le cloud mais aucun fichier local n'a été détecté.",
    };
  }

  if (!remoteDate) {
    return {
      label: "Jamais sync",
      variant: "warning",
      Icon: Cloud,
      description:
        "La sauvegarde locale existe mais n'a jamais été envoyée sur le cloud.",
    };
  }

  const timeDiff = localDate.getTime() - remoteDate.getTime();

  if (Math.abs(timeDiff) < 60000) {
    return {
      label: "À jour",
      variant: "success",
      Icon: CheckCircle2,
      description: "Vos sauvegardes locales et cloud sont synchronisées.",
    };
  }

  if (localDate > remoteDate) {
    return {
      label: "Local plus récent",
      variant: "info",
      Icon: CloudUpload,
      description:
        "Votre sauvegarde locale est plus récente que celle présente sur le cloud.",
    };
  }

  return {
    label: "Cloud plus récent",
    variant: "warning",
    Icon: CloudDownload,
    description:
      "La version sur le cloud est plus récente que votre sauvegarde locale.",
  };
}
