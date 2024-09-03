"use client";

import { dialog, invoke } from "@tauri-apps/api";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Store } from "tauri-plugin-store-api";
import { useAtom } from "jotai";
import { directoriesAtom } from "@/lib/atoms";
import { DATA_FILE } from "@/lib/constants";

const store = new Store(DATA_FILE);

const SelectDirectory = () => {
  const [directories, setDirectories] = useAtom<string[]>(directoriesAtom);

  useEffect(() => {
    console.log("Saving directories ", directories);
    store.set("directories", { directories: directories });
    store.save();

    invoke("index", { userSuppliedPaths: directories })
      .then(console.log)
      .catch(console.error);
  }, [directories]);

  const handleSelectDirectory = async () => {
    console.log("Selecting directory");
    const filePaths = await dialog.open({ directory: true });

    if (Array.isArray(filePaths)) {
      console.log("Multiple files selected");
      filePaths.forEach((filePath) => {
        if (!directories.includes(filePath)) {
          directories.push(filePath);
        }
      });
      setDirectories([...directories]);
    } else if (filePaths === null) {
      // user cancelled the selection
      console.log("User cancelled the selection");
    } else {
      // user selected a single file
      console.log("Single file selected");
      if (!directories.includes(filePaths)) {
        directories.push(filePaths);
      }
      setDirectories([...directories]);
    }
  };

  return (
    <div>
      <Button onClick={handleSelectDirectory}>Index Folder</Button>
    </div>
  );
};

export default SelectDirectory;
