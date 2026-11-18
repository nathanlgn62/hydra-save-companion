import { getReportAuthToken } from "./cloud-token";

export async function getGoogleIdToken(): Promise<string> {
  const result = await getReportAuthToken();
  return result.token;
}
