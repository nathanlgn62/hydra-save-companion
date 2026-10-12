export const getStoredStatus = () => {
  const cloudProvider = localStorage.getItem("cloud_provider");
  const cloudToken = localStorage.getItem("cloud_token");
  const gdriveToken = localStorage.getItem("gdrive_token");

  const isConnected = !!cloudToken || !!gdriveToken;
  const provider = cloudProvider || (gdriveToken ? "google-drive" : null);

  return {
    isConnected,
    provider: isConnected ? provider : null,
  };
};
