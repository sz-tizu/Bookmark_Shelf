import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { vi, describe, it, expect, beforeEach } from "vitest";
import { useConfig } from "../hooks/useConfig";

const mockInvoke = vi.mocked(invoke);

const DEFAULT_CONFIG = {
  general: { bookmark_dir: "/home/user/bookmarks" },
  checker: { concurrency: 20, timeout_secs: 10, follow_redirects: true },
};

describe("useConfig", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("loads config on mount", async () => {
    mockInvoke.mockResolvedValueOnce(DEFAULT_CONFIG);

    const { result } = renderHook(() => useConfig());

    await waitFor(() => {
      expect(result.current.config).not.toBeNull();
    });

    expect(result.current.config?.general.bookmark_dir).toBe("/home/user/bookmarks");
    expect(result.current.config?.checker.concurrency).toBe(20);
    expect(mockInvoke).toHaveBeenCalledWith("get_config");
  });

  it("save calls invoke with updated config", async () => {
    mockInvoke.mockResolvedValueOnce(DEFAULT_CONFIG); // initial load
    mockInvoke.mockResolvedValueOnce(undefined);      // save

    const { result } = renderHook(() => useConfig());
    await waitFor(() => expect(result.current.config).not.toBeNull());

    const updated = {
      ...DEFAULT_CONFIG,
      checker: { ...DEFAULT_CONFIG.checker, concurrency: 42 },
    };
    await act(async () => {
      await result.current.save(updated);
    });

    expect(mockInvoke).toHaveBeenCalledWith("save_config", { config: updated });
    expect(result.current.config?.checker.concurrency).toBe(42);
  });

  it("config remains null when invoke rejects", async () => {
    mockInvoke.mockRejectedValueOnce(new Error("backend error"));

    const { result } = renderHook(() => useConfig());

    // Wait a tick — config should stay null on error
    await new Promise((r) => setTimeout(r, 50));
    expect(result.current.config).toBeNull();
  });
});
