import { atom } from "jotai";
import { Store } from "tauri-plugin-store-api";

import { StoredDirectories } from "./types";

export const directoriesAtom = atom<string[]>([]);
