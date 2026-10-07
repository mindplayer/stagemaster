import type { ApplicationHost } from "../../application-host";
import type { InstallationController } from "../installation/useInstallation";
import type { DeliveryNavigation } from "../installation/useDeliveryNavigation";
import { InstallationCenter } from "../installation/InstallationCenter";
import { DeviceCenter } from "./DeviceCenter";
export function DeviceTools({
  host,
  installation,
  delivery,
  canPrepare,
}: {
  host: ApplicationHost;
  installation: InstallationController;
  delivery: DeliveryNavigation;
  canPrepare: boolean;
}) {
  return (
    <>
      <DeviceCenter
        host={host}
        dismiss={installation.open}
        onOpen={() => installation.setOpen(false)}
        revealRequest={delivery.deviceReveal}
        onInstallation={() => installation.setOpen(true)}
      />
      <InstallationCenter
        controller={installation}
        canPrepare={canPrepare}
        onPreparePackage={delivery.preparePackage}
        onManageDevice={delivery.openDevice}
      />
    </>
  );
}
