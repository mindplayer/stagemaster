import type { ApplicationHost } from "../../application-host";
import type { InstallationController } from "../installation/useInstallation";
import { InstallationCenter } from "../installation/InstallationCenter";
import { DeviceCenter } from "./DeviceCenter";
export function DeviceTools({
  host,
  installation,
}: {
  host: ApplicationHost;
  installation: InstallationController;
}) {
  return (
    <>
      <DeviceCenter
        host={host}
        dismiss={installation.open}
        onOpen={() => installation.setOpen(false)}
      />
      <InstallationCenter controller={installation} />
    </>
  );
}
