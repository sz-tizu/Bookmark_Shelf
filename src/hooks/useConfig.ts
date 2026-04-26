import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Config {
  general: { bookmark_dir: string };
  checker: { concurrency: number; timeout_secs: number; follow_redirects: boolean };
}

export function useConfig() {
  const [config, setConfig] = useState<Config | null>(null);

  useEffect(() => {
    invoke<Config>("get_config").then(setConfig).catch(console.error);
  }, []);

  const save = async (next: Config) => {
    await invoke("save_config", { config: next });
    setConfig(next);
  };

  return { config, save };
}
