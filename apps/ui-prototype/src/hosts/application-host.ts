import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ApplicationHost } from "../application-host";
export const applicationHost: ApplicationHost = isTauri()
  ? {
      kind: "desktop",
      check: (generation) => invoke("check_request", { generation }),
      preview: (request) => invoke("preview_request", { request }),
      previs: (request) => invoke("previs_request", { request }),
      request: (request) => invoke("project_request", { request }),
      onCloseRequested: (handler) => listen("project-close-requested", handler),
    }
  : {
      kind: "browser",
      check: async () => {
        throw new Error("请使用桌面应用检查工程");
      },
      preview: async () => {
        throw new Error("请使用桌面应用预览");
      },
      previs: async () => {
        throw new Error("请使用桌面应用打开三维预演");
      },
      request: async () => {
        throw new Error("请使用桌面应用打开本地工程");
      },
      onCloseRequested: async () => () => {},
    };
