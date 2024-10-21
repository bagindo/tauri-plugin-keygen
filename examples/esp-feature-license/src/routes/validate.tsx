import {
  type KeygenLicense,
  KeygenError,
  validateKey,
  getLicenseKey,
} from "tauri-plugin-keygen-api";
import { z } from "zod";
import { useState } from "react";
import { useAtom } from "jotai";
import { createFileRoute, useRouter } from "@tanstack/react-router";
import { getLicenseErrMessage } from "../utils";
import { proLicenseModalAtom } from "../atoms";

export const Route = createFileRoute("/validate")({
  validateSearch: z.object({
    redirect: z.string().optional().catch(""),
    err: z.string().optional().catch(""),
  }),
  loader: async () => {
    const licenseKey = await getLicenseKey();
    return { licenseKey };
  },
  component: () => <Validate />,
});

function Validate() {
  const router = useRouter();
  const navigate = Route.useNavigate();
  const { redirect, err: errParam } = Route.useSearch();
  const { licenseKey } = Route.useLoaderData();

  const [_, setProLicenseDialogOpened] = useAtom(proLicenseModalAtom);
  const [key, setKey] = useState(licenseKey || "");
  const [loading, setLoading] = useState(false);
  const [err, setErr] = useState(errParam || "");

  const validate = async () => {
    if (key === "") {
      setErr("License can't be empty");
      return;
    }

    if (key.length < 37) {
      setErr("Invalid license");
      return;
    }

    setErr("");
    setLoading(true);

    let license: KeygenLicense;

    try {
      license = await validateKey({ key, entitlements: ["ADD_IMAGE"] });
    } catch (e) {
      if (e instanceof KeygenError) {
        const { code, detail } = e;
        setErr(getLicenseErrMessage({ code, detail }));
      } else {
        setErr("Unknown error occured");
        console.error(e);
      }

      setLoading(false);
      return;
    }

    if (license.valid) {
      setProLicenseDialogOpened(false);
      await router.invalidate();
      await navigate({ to: redirect || "/" });
    } else {
      console.log(license);

      setErr(
        getLicenseErrMessage({
          code: license.code,
          detail: license.detail,
          policyId: license.policyId,
        }),
      );
    }

    setLoading(false);
  };

  return (
    <div className="flex h-screen w-screen cursor-default flex-col justify-center bg-app-red px-6 pb-14 text-app-white antialiased">
      {/* Header */}
      <h1 className="select-none font-syne text-5xl font-bold">Daily ESP</h1>
      <p className="mt-2.5 select-none text-sm font-medium tracking-wide">
        Enter your license
      </p>

      {/* License Key Input */}
      <div className="mt-12">
        <label
          htmlFor="license-key"
          className="select-none text-sm font-medium tracking-wide"
        >
          License Key
        </label>
        <input
          autoFocus
          id="license-key"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          className="mt-2.5 block h-12 w-full rounded-md border border-app-white/60 bg-transparent px-3 uppercase tracking-wider transition-colors focus:border-app-white focus:outline-none"
        />
      </div>

      {/* Validate Button */}
      <button
        onClick={validate}
        disabled={loading}
        className="mb-3 mt-6 h-11 w-full select-none rounded-md bg-app-white/90 text-center text-sm font-medium uppercase tracking-wider text-stone-900 transition-all hover:bg-white active:bg-app-white/90 disabled:bg-app-white/60 disabled:hover:bg-app-white/60 disabled:active:bg-app-white/60"
      >
        Validate
      </button>

      {/* Loading / Err */}
      <div className="flex h-10 w-full cursor-default select-none items-center justify-start">
        {loading && <div>validating license...</div>}
        {err !== "" && <div>{err}</div>}
      </div>
    </div>
  );
}
