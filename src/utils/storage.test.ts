import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { getStoredStatus } from "./storage";

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

describe("storage utils", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it("should return disconnected state when localStorage is empty", () => {
    const status = getStoredStatus();
    expect(status.isConnected).toBe(false);
    expect(status.provider).toBeNull();
  });

  it("should return Dropbox provider when cloud_provider is dropbox and cloud_token exists", () => {
    localStorage.setItem("cloud_provider", "dropbox");
    localStorage.setItem("cloud_token", "test-dropbox-token");

    const status = getStoredStatus();
    expect(status.isConnected).toBe(true);
    expect(status.provider).toBe("dropbox");
  });

  it("should return Google Drive provider when cloud_provider is google-drive", () => {
    localStorage.setItem("cloud_provider", "google-drive");
    localStorage.setItem("cloud_token", "test-gdrive-token");

    const status = getStoredStatus();
    expect(status.isConnected).toBe(true);
    expect(status.provider).toBe("google-drive");
  });

  it("should support legacy gdrive_token fallback", () => {
    localStorage.setItem("gdrive_token", "legacy-token");

    const status = getStoredStatus();
    expect(status.isConnected).toBe(true);
    expect(status.provider).toBe("google-drive");
  });

  it("should not be connected if cloud_provider is set without any token", () => {
    localStorage.setItem("cloud_provider", "dropbox");

    const status = getStoredStatus();
    expect(status.isConnected).toBe(false);
    expect(status.provider).toBeNull();
  });
});
