import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getReportAuthToken } from "./cloud-token";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockedInvoke = vi.mocked(invoke);

const storageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(globalThis, "localStorage", {
  value: storageMock,
  writable: true,
});

describe("cloud-token utils", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("should throw if user is not connected", async () => {
    await expect(getReportAuthToken()).rejects.toThrow(
      "You must be signed in with a Google or Dropbox account to perform this action.",
    );
  });

  it("should return token and provider for google-drive", async () => {
    localStorage.setItem("cloud_provider", "google-drive");
    localStorage.setItem("cloud_token", "access|refresh|id_token");

    mockedInvoke.mockResolvedValueOnce({
      token: "valid_google_id_token",
      provider: "google-drive",
      updatedCloudToken: null,
    });

    const res = await getReportAuthToken();
    expect(res.token).toBe("valid_google_id_token");
    expect(res.provider).toBe("google-drive");
    expect(mockedInvoke).toHaveBeenCalledWith("get_cloud_auth_token", {
      token: "access|refresh|id_token",
      provider: "google-drive",
    });
  });

  it("should return token and provider for dropbox", async () => {
    localStorage.setItem("cloud_provider", "dropbox");
    localStorage.setItem("cloud_token", "dropbox_access|dropbox_refresh");

    mockedInvoke.mockResolvedValueOnce({
      token: "valid_dropbox_access_token",
      provider: "dropbox",
      updatedCloudToken: "new_dropbox_access|dropbox_refresh",
    });

    const res = await getReportAuthToken();
    expect(res.token).toBe("valid_dropbox_access_token");
    expect(res.provider).toBe("dropbox");
    expect(mockedInvoke).toHaveBeenCalledWith("get_cloud_auth_token", {
      token: "dropbox_access|dropbox_refresh",
      provider: "dropbox",
    });
    expect(localStorage.getItem("cloud_token")).toBe(
      "new_dropbox_access|dropbox_refresh",
    );
  });
});
