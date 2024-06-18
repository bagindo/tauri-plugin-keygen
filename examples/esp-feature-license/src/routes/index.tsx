import { createFileRoute, defer, Navigate } from "@tanstack/react-router";
import { formatISO } from "date-fns";
import {
  getLicenseKey,
  getLicense,
  validateCheckoutKey,
} from "tauri-plugin-keygen-api";

export const Route = createFileRoute("/")({
  loader: () => {
    const backgroundValidation = async () => {
      const licenseKey = await getLicenseKey();
      const license = await getLicense();

      if (license === null && licenseKey !== null) {
        const license = await validateCheckoutKey({
          key: licenseKey,
          ttlSeconds: 25200 /* 1 week */,
        });

        return license;
      }

      return license;
    };

    // make the request to set the state on the back-end (Tauri App State)
    const license = backgroundValidation();

    return {
      license: defer(license), // we're not gonna use this on the front-end
    };
  },
  component: () => (
    <Navigate
      to="/esp"
      search={{ date: formatISO(new Date(), { representation: "date" }) }}
    />
  ),
});
