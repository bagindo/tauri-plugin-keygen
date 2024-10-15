// Tauri APIs
import { open as openLink } from "@tauri-apps/plugin-shell";
// React
import * as Dialog from "@radix-ui/react-dialog";
import { Link, useLocation } from "@tanstack/react-router";
import { useAtom } from "jotai";
import { XMarkIcon } from "@heroicons/react/20/solid";
import { proLicenseModalAtom } from "../atoms";

export default function ProLicenseModal() {
  const location = useLocation();
  const [modalOpened, setModalOpened] = useAtom(proLicenseModalAtom);

  return (
    <Dialog.Root open={modalOpened} onOpenChange={setModalOpened}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 cursor-default select-none bg-black/50" />
        <Dialog.Content className="fixed left-[50%] top-[50%] max-h-[85vh] w-[90vw] max-w-[450px] translate-x-[-50%] translate-y-[-50%] rounded-[6px] bg-app-white px-4 py-3 shadow-lg focus:outline-none">
          <Dialog.Title className="cursor-default select-none text-lg font-semibold capitalize text-stone-800">
            Pro Feature
          </Dialog.Title>

          {/* License Input */}
          <div className="mt-3 cursor-default select-none truncate rounded-md border border-app-red/20 bg-app-red/10 p-2.5 text-app-red">
            This is a pro feature
          </div>

          {/* Validate Button */}
          <div className="mt-6 flex items-center justify-end space-x-2">
            <Link
              to="/validate"
              search={{ redirect: location.href }}
              className="select-none rounded-md bg-stone-300/80 px-3 py-2 text-center text-xs font-medium uppercase tracking-wider text-stone-800 transition-all hover:bg-stone-300/60 focus:outline-none active:bg-stone-300/80"
            >
              Enter License
            </Link>
            <button
              onClick={() => {
                openLink("https://www.stripe.com");
              }}
              className="select-none rounded-md bg-stone-800 px-3 py-2 text-center text-xs font-medium uppercase tracking-wider text-app-white transition-all hover:bg-stone-800/90 focus:outline-none active:bg-stone-800"
            >
              Buy Pro
            </button>
          </div>

          <Dialog.Close asChild>
            <button
              className="absolute right-2.5 top-2.5 inline-flex h-6 w-6 items-center justify-center rounded-full transition-all hover:bg-stone-200 focus:outline-none focus:ring-1 focus:ring-stone-400 active:bg-stone-100"
              aria-label="Close"
            >
              <XMarkIcon className="size-4" />
            </button>
          </Dialog.Close>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
