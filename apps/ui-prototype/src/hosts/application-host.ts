import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ApplicationHost } from "../application-host";
export const applicationHost: ApplicationHost = isTauri()
  ? {
      kind: "desktop",
      execution: (request) => invoke("execution_request", { request }),
      importEffectTemplate: (generation, sceneId, fixtureIds) =>
        invoke("effect_template_import", { generation, sceneId, fixtureIds }),
      exportEffectTemplate: (generation, sceneId, effectId) =>
        invoke("effect_template_export", { generation, sceneId, effectId }),
      cancelEffectTemplate: (token) =>
        invoke("effect_template_cancel", { token }),
      importProfile: (generation) =>
        invoke("profile_file_import", { generation }),
      exportProfile: (generation, profileId) =>
        invoke("profile_file_export", { generation, profileId }),
      exportSequenceReport: (generation, sequenceId) =>
        invoke("sequence_report_export", { generation, sequenceId }),
      exportPatchReport: (generation) =>
        invoke("patch_report_export", { generation }),
      output: (request) => invoke("output_request", { request }),
      recent: (request) => invoke("recent_request", { request }),
      audioPrepare: (generation, kind) =>
        invoke("audio_prepare", { generation, kind }),
      audioCancel: () => invoke("audio_cancel"),
      audio: (generation, command) =>
        invoke("audio_request", { generation, command }),
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
      execution: async () => { throw new Error("请使用桌面应用管理本机后台执行"); },
      importEffectTemplate: async () => {
        throw new Error("请使用桌面应用导入灯效模板");
      },
      exportEffectTemplate: async () => {
        throw new Error("请使用桌面应用导出灯效模板");
      },
      cancelEffectTemplate: async () => {},
      importProfile: async () => {
        throw new Error("请使用桌面应用导入本机灯具模式");
      },
      exportProfile: async () => {
        throw new Error("请使用桌面应用导出灯具模式");
      },
      exportSequenceReport: async () => {
        throw new Error("请使用桌面应用导出节目单");
      },
      exportPatchReport: async () => {
        throw new Error("请使用桌面应用导出配灯表");
      },
      output: async () => {
        throw new Error("请使用桌面应用控制预演亮度");
      },
      recent: async () => [],
      audioPrepare: async () => {
        throw new Error("请使用桌面应用导入本机音乐");
      },
      audioCancel: async () => {},
      audio: async () => {
        throw new Error("请使用桌面应用试听音乐");
      },
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
