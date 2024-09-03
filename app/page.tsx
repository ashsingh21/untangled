"use client";

import React, { useEffect } from "react";

import dynamic from "next/dynamic";
import { useSetAtom } from "jotai";
import { directoriesAtom } from "@/lib/atoms";
import { StoredDirectories } from "@/lib/types";
import { Store } from "tauri-plugin-store-api";
import { DATA_FILE } from "@/lib/constants";

const Search = dynamic(() => import("@/components/search"), {
  ssr: false,
});

const store = new Store(DATA_FILE);

export default function Home() {
  const setDirectories = useSetAtom(directoriesAtom);

  useEffect(() => {
    store
      .get<StoredDirectories>("directories")
      .then((dirs) => {
        if (!dirs) return;
        setDirectories(dirs.directories);
      })
      .catch((e) => {
        console.error(e);
      });
  }, [setDirectories]);

  return (
    <main className="flex min-h-screen flex-col justify-between p-10">
      <div className="">
        <Search />
      </div>
    </main>
  );
}
