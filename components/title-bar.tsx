"use client";

import { useEffect } from "react";
import { appWindow } from "@tauri-apps/api/window";

export default function TitleBar() {
  useEffect(() => {
    const minimizeButton = document.getElementById("titlebar-minimize");
    const maximizeButton = document.getElementById("titlebar-maximize");
    const closeButton = document.getElementById("titlebar-close");

    if (minimizeButton && maximizeButton && closeButton) {
      minimizeButton.addEventListener("click", () => appWindow.minimize());
      maximizeButton.addEventListener("click", () =>
        appWindow.toggleMaximize(),
      );
      closeButton.addEventListener("click", () => appWindow.close());
    }

    return () => {
      // Cleanup event listeners on component unmount
      if (minimizeButton)
        minimizeButton.removeEventListener("click", () => appWindow.minimize());
      if (maximizeButton)
        maximizeButton.removeEventListener("click", () =>
          appWindow.toggleMaximize(),
        );
      if (closeButton)
        closeButton.removeEventListener("click", () => appWindow.close());
    };
  }, []); // Empty dependency array ensures this runs only once on mount

  return (
    <div data-tauri-drag-region className="titlebar rounded-xl">
      <div className="titlebar-button" id="titlebar-minimize">
        <img
          src="https://api.iconify.design/mdi:window-minimize.svg"
          alt="minimize"
        />
      </div>
      <div className="titlebar-button" id="titlebar-maximize">
        <img
          src="https://api.iconify.design/mdi:window-maximize.svg"
          alt="maximize"
        />
      </div>
      <div className="titlebar-button" id="titlebar-close">
        <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
      </div>
    </div>
  );
}
