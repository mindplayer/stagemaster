import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ApplicationHost } from "../application-host";
export const applicationHost: ApplicationHost = isTauri()
  ? {
      kind: "desktop",
      installation: (request) => invoke("installation_request", { request }),
      startInstallation: (generation, token, epoch, deviceId) =>
        invoke("installation_start", { generation, token, epoch, deviceId }),
      device: (request) => invoke("device_request", { request }),
      recovery: (request) => invoke("recovery_request", { request }),
      buildPackage: (generation, selection) =>
        invoke("package_build", { generation, selection }),
      exportPackage: (generation, token) =>
        invoke("package_export", { generation, token }),
      check: (generation) => invoke("check_request", { generation }),
      preview: (request) => invoke("preview_request", { request }),
      previs: (request) => invoke("previs_request", { request }),
      request: (request) => invoke("project_request", { request }),
      onCloseRequested: (handler) => listen("project-close-requested", handler),
    }
  : {
      kind: "browser",
      installation: async () => {
        throw new Error("请使用桌面应用管理设备节目安装");
      },
      startInstallation: async () => {
        throw new Error("请使用桌面应用安装设备节目");
      },
      device: async () => {
        throw new Error("请使用桌面应用连接蓝牙设备");
      },
      recovery: async () => {
        throw new Error("请使用桌面应用恢复工程");
      },
      buildPackage: async () => {
        throw new Error("请使用桌面应用生成播放包");
      },
      exportPackage: async () => {
        throw new Error("请使用桌面应用导出播放包");
      },
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
