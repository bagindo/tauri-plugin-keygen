import { Suspense } from "react";
import {
  createFileRoute,
  defer,
  redirect,
  Await,
  Link,
  Navigate,
  Outlet,
} from "@tanstack/react-router";
import {
  getLicenseKey,
  getLicense,
  validateCheckoutKey,
  KeygenError,
  KeygenLicense,
} from "tauri-plugin-keygen-api";
import { intervalToDuration } from "date-fns";
import * as Dialog from "@radix-ui/react-dialog";
import { getPolicyName, getLicenseErrMessage } from "../utils";

export const Route = createFileRoute("/_licensed")({
  beforeLoad: async ({ location }) => {
    const licenseKey = await getLicenseKey();

    if (licenseKey === null) {
      throw redirect({
        to: "/validate",
        search: {
          redirect: location.href,
        },
      });
    }
  },
  loader: () => {
    const backgroundValidation = async () => {
      const licenseKey = await getLicenseKey();
      const license = await getLicense();

      if (license === null) {
        const license = await validateCheckoutKey({
          key: licenseKey || "",
          ttlSeconds: 25200 /* 1 week */,
        });

        return license;
      }

      return license;
    };

    const license = backgroundValidation();

    return {
      license: defer(license),
    };
  },
  component: Licensed,
  errorComponent: ({ error }) => {
    let err: string;

    if (error instanceof KeygenError) {
      const { code, detail } = error;
      err = getLicenseErrMessage({ code, detail });
    } else {
      err = error.message;
    }

    return <Navigate to="/validate" search={{ err }} />;
  },
});

function Licensed() {
  const { license } = Route.useLoaderData();

  return (
    <>
      <Suspense>
        <Await promise={license}>
          {(license) => {
            if (!license.valid) {
              return <LicenseErrDialog license={license} />;
            }

            // update the PolicyId in getPolicyName function in utils.ts
            // then un-comment the following IF block to make sure the remaining TRIAL badge
            // is only shown for your trial license

            // if (getPolicyName(license.policyId) === "trial") {
            const remainingTrial = intervalToDuration({
              start: new Date(),
              end: new Date(license.expiry || ""),
            });

            return (
              <div className="fixed bottom-3 left-3 z-50 cursor-default select-none rounded-full bg-stone-800 px-2.5 py-1 text-xs uppercase tracking-wide text-app-white">
                {remainingTrial.days} days trial
              </div>
            );
            // }

            return null;
          }}
        </Await>
      </Suspense>
      <Outlet />
    </>
  );
}

function LicenseErrDialog({ license }: { license: KeygenLicense }) {
  return (
    <Dialog.Root open={true}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 cursor-default select-none bg-black/50" />
        <Dialog.Content className="fixed left-[50%] top-[50%] max-h-[85vh] w-[90vw] max-w-[450px] translate-x-[-50%] translate-y-[-50%] rounded-[6px] bg-app-white px-4 py-3 shadow-lg focus:outline-none">
          <Dialog.Title className="cursor-default select-none text-lg font-semibold capitalize text-stone-800">
            Invalid License
          </Dialog.Title>

          {/* License Input */}
          <div className="mt-3 cursor-default select-none truncate rounded-md border border-app-red/20 bg-app-red/10 p-2.5">
            {license.key}
          </div>

          {/* Error detail */}
          <p className="mt-1.5 cursor-default select-none text-sm text-app-red">
            {getLicenseErrMessage({
              code: license.code,
              detail: license.detail,
              policyId: license.policyId,
            })}
          </p>

          {/* Validate Button */}
          <div className="mt-6 flex items-center justify-end space-x-2">
            <Link
              to="/validate"
              className="select-none rounded-md bg-stone-300/80 px-3 py-2 text-center text-xs font-medium uppercase tracking-wider text-stone-800 transition-all hover:bg-stone-300/60 focus:outline-none active:bg-stone-300/80"
            >
              New License
            </Link>
            <button
              onClick={() => {}}
              className="select-none rounded-md bg-stone-800 px-3 py-2 text-center text-xs font-medium uppercase tracking-wider text-app-white transition-all hover:bg-stone-800/90 focus:outline-none active:bg-stone-800"
            >
              Buy Pro
            </button>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
