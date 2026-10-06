export const getProviderDisplayName = (provider: string | null) => {
  switch (provider) {
    case "google-drive":
      return "Google Drive";
    case "dropbox":
      return "Dropbox";
    case "mega":
      return "Mega";
    case "proton-drive":
      return "Proton Drive";
    default:
      return "Cloud connecté";
  }
};
