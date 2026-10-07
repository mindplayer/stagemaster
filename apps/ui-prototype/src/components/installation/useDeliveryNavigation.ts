import { useState } from "react";

/** Navigation never changes a connection, installation or execution state. */
export function useDeliveryNavigation(
  preparePackage: () => Promise<boolean>,
  closeInstallation: () => void,
) {
  const [packageFocus, setPackageFocus] = useState(0);
  const [deviceReveal, setDeviceReveal] = useState(0);
  return {
    packageFocus,
    deviceReveal,
    async preparePackage() {
      if (await preparePackage()) {
        closeInstallation();
        setPackageFocus((value) => value + 1);
      }
    },
    openDevice() {
      closeInstallation();
      setDeviceReveal((value) => value + 1);
    },
  };
}
export type DeliveryNavigation = ReturnType<typeof useDeliveryNavigation>;
