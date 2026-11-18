import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { getGoogleIdToken } from "./google-token";

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

describe("google-token utils", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("should throw if user is not connected", async () => {
    await expect(getGoogleIdToken()).rejects.toThrow(
      "You must be signed in with a Google or Dropbox account to perform this action.",
    );
  });

  it("should return idToken when connected to google-drive", async () => {
    localStorage.setItem("cloud_provider", "google-drive");
    localStorage.setItem("cloud_token", "access|refresh|existing_id_token");

    mockedInvoke.mockResolvedValueOnce({
      token: "valid_id_token",
      provider: "google-drive",
      updatedCloudToken: null,
    });

    const token = await getGoogleIdToken();
    expect(token).toBe("valid_id_token");
    expect(mockedInvoke).toHaveBeenCalledWith("get_cloud_auth_token", {
      token: "access|refresh|existing_id_token",
      provider: "google-drive",
    });
  });

  it("should update localStorage when updatedCloudToken is returned", async () => {
    localStorage.setItem("cloud_provider", "google-drive");
    localStorage.setItem("cloud_token", "old_access|refresh|expired_id");

    mockedInvoke.mockResolvedValueOnce({
      token: "fresh_id_token",
      provider: "google-drive",
      updatedCloudToken: "new_access|refresh|fresh_id_token",
    });

    const token = await getGoogleIdToken();
    expect(token).toBe("fresh_id_token");
    expect(localStorage.getItem("cloud_token")).toBe(
      "new_access|refresh|fresh_id_token",
    );
  });
});
