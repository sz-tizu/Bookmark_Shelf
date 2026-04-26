import "@testing-library/jest-dom";
import { vi } from "vitest";

// Mock the entire Tauri API — tests run in jsdom, not a real Tauri window.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-fs", () => ({
  writeTextFile: vi.fn(),
  readTextFile: vi.fn(),
}));
