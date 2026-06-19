import { getCurrentWindow } from "@tauri-apps/api/window";

export async function setTauriWindowTitle(title: string): Promise<boolean> {
  if (!("__TAURI_INTERNALS__" in window)) return false;
  try {
    await getCurrentWindow().setTitle(title);
    return true;
  } catch {
    return false;
  }
}
