export const getStoredStatus = () => {
  const currentProvider =
    localStorage.getItem("cloud_provider") ||
    localStorage.getItem("gdrive_token")
      ? "google-drive"
      : null;
  const isConnected =
    !!localStorage.getItem("cloud_token") ||
    !!localStorage.getItem("gdrive_token") ||
    !!localStorage.getItem("cloud_provider");
  return { isConnected, provider: currentProvider };
};
