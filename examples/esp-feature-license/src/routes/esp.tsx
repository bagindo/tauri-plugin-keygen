// Tauri
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import { remove } from "@tauri-apps/plugin-fs";
import { getLicense } from "tauri-plugin-keygen-api";

// React
import { forwardRef, useEffect, useState } from "react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { z } from "zod";
import { nanoid } from "nanoid";
import { useAtom } from "jotai";
import {
  ArrowLeftIcon,
  ArrowRightIcon,
  ChevronDownIcon,
  PlusIcon,
  TrashIcon,
} from "@heroicons/react/20/solid";
import { PhotoIcon } from "@heroicons/react/24/outline";
import DatePicker from "react-datepicker";
import "react-datepicker/dist/react-datepicker.min.css";
import { addDays, format, formatISO } from "date-fns";

// App
import type { ESPItem, ESPType } from "../types";
import { getESPItems, upsertESPItem, deleteESPItem } from "../api/esps";
import { saveImageToAppData } from "../api/images";
import { proLicenseModalAtom } from "../atoms";
import ProLicenseModal from "../components/ProLicenseModal";

// Router
const espSearchSchema = z.object({
  date: z.string().date(),
});

export const Route = createFileRoute("/esp")({
  validateSearch: espSearchSchema,
  loaderDeps: ({ search: { date } }) => ({ date }),
  loader: async ({ deps: { date } }) => getESPItems({ date }),
  component: ESP,
  shouldReload: true,
  gcTime: 0,
});

const espTypes: ESPType[] = ["effort", "success", "progress"];

function ESP() {
  const navigate = useNavigate({ from: Route.fullPath });

  const { date } = Route.useSearch();

  const espItems = Route.useLoaderData();

  return (
    <div className="flex h-screen flex-col antialiased">
      {/* Header */}
      <div className="flex h-28 w-full select-none items-center bg-app-red px-6 pb-1 text-white">
        {/* Current Day */}
        <div className="flex-1">
          {/* Month Year Label */}
          <h1 className="flex items-center space-x-2 text-3xl">
            <span className="font-bold">{format(date, "MMM")}</span>
            <span>{format(date, "yyyy")}</span>
          </h1>

          {/* Date Picker */}
          <DatePicker
            selected={new Date(date)}
            onChange={(date) => {
              if (date !== null) {
                navigate({
                  search: { date: formatISO(date, { representation: "date" }) },
                });
              }
            }}
            customInput={<DatePickerButton />}
            calendarStartDay={1}
            popperPlacement="bottom-start"
          />
        </div>

        {/* Go to Date */}
        <div className="flex shrink-0 items-center space-x-2.5">
          {/* Go to: Today */}
          <button
            className="h-6 rounded-full border border-white/25 bg-white/10 px-2 text-xs font-medium transition-colors hover:bg-white/20 disabled:hidden"
            onClick={() => {
              navigate({
                search: () => ({
                  date: formatISO(new Date(), { representation: "date" }),
                }),
              });
            }}
            disabled={
              formatISO(new Date(), { representation: "date" }) ===
              formatISO(date, { representation: "date" })
            }
          >
            Today
          </button>

          {/* Go to: Prev Day */}
          <button
            className="flex size-8  shrink-0 items-center justify-center rounded-full bg-app-white/95 transition-colors hover:bg-white active:bg-app-white/85"
            onClick={() => {
              navigate({
                search: (prev) => ({
                  date: formatISO(addDays(prev.date, -1), {
                    representation: "date",
                  }),
                }),
              });
            }}
          >
            <ArrowLeftIcon className="size-5 text-stone-900" />
          </button>

          {/* Go to: Next Day */}
          <button
            className="flex size-8 shrink-0 items-center justify-center rounded-full bg-app-white/95 transition-colors hover:bg-white active:bg-app-white/85"
            onClick={() => {
              navigate({
                search: (prev) => ({
                  date: formatISO(addDays(prev.date, 1), {
                    representation: "date",
                  }),
                }),
              });
            }}
          >
            <ArrowRightIcon className="size-5 text-stone-900" />
          </button>
        </div>
      </div>

      {/* ESP Content */}
      <div className="custom-scroll-bar w-full flex-1 overflow-auto px-6 pt-6">
        {espTypes.map((espType) => (
          <ESPSection
            key={`${espType}-${date}`}
            date={date}
            type={espType}
            items={espItems.filter((item) => item.type === espType)}
          />
        ))}
      </div>

      {/* Pro License Modal */}
      <ProLicenseModal />
    </div>
  );
}

type DatePickerButtonProps = {
  value?: string;
  onClick?: () => void;
};

const DatePickerButton = forwardRef<HTMLButtonElement, DatePickerButtonProps>(
  ({ value, onClick }, ref) => {
    const date = value === undefined ? new Date() : new Date(value);

    return (
      <button
        ref={ref}
        onClick={onClick}
        className="mt-0.5 flex items-center space-x-2 text-xl"
      >
        <span>{format(date, "eeee, d")}</span>

        <ChevronDownIcon className="size-5 text-white/80" />
      </button>
    );
  },
);

type ESPSectionProps = {
  date: string;
  type: ESPType;
  items: ESPItem[];
};

const MINIMUM_ITEM_COUNTS = 3;

const initSectionItems = ({
  type,
  items,
}: {
  type: ESPType;
  items: ESPItem[];
}) => {
  return [
    ...items,
    ...getEmptyItems({ type, count: MINIMUM_ITEM_COUNTS - items.length }),
  ];
};

const getEmptyItems = ({ type, count }: { type: ESPType; count: number }) => {
  const emptyItems: ESPItem[] = [];

  for (let i = 0; i < count; i++) {
    emptyItems.push(getEmptyItem(type));
  }

  return emptyItems;
};

const getEmptyItem = (type: ESPType): ESPItem => ({
  id: nanoid(),
  type,
  content: "",
  image: "",
});

function ESPSection({ date, type, items }: ESPSectionProps) {
  const [sectionItems, setSectionItems] = useState<ESPItem[]>(
    initSectionItems({ type, items }),
  );

  useEffect(() => {
    setSectionItems(initSectionItems({ type, items }));
  }, [items]);

  const onItemUpdated = (item: ESPItem) => {
    // update section items
    setSectionItems((prev) =>
      prev.map((sectionItem) =>
        sectionItem.id === item.id ? item : sectionItem,
      ),
    );

    // persist updated item
    if (item.content === "" && item.image === "") {
      deleteESPItem({ id: item.id, date }).catch((_) =>
        console.error("failed deleting ESP item"),
      );
    } else {
      upsertESPItem({ date, espItem: item }).catch((_) =>
        console.error("failed upserting ESP item"),
      );
    }
  };

  return (
    <div className="mb-10 text-stone-900">
      <h2 className="font-syne text-2xl font-bold capitalize">{type}</h2>

      {/* Section Items */}
      {sectionItems.map((item) => (
        <ESPItemInput
          key={`${date}-${item.id}`}
          date={date}
          item={item}
          onItemUpdated={onItemUpdated}
        />
      ))}

      {/* Add New Button */}
      <button
        className="group flex h-14 w-full select-none items-center space-x-1 border-b border-b-transparent text-sm text-stone-400 focus:border-b-stone-800 focus:outline-none"
        onClick={() => setSectionItems((prev) => [...prev, getEmptyItem(type)])}
      >
        <PlusIcon className="size-3.5" />
        <span className="block group-focus:hidden">New</span>
        <span className="hidden group-focus:block">Press enter to add new</span>
      </button>
    </div>
  );
}

function ESPItemInput({
  date,
  item,
  onItemUpdated,
}: {
  date: string;
  item: ESPItem;
  onItemUpdated: (item: ESPItem) => void;
}) {
  const [value, setValue] = useState(item.content);

  return (
    <div key={`${date}-${item.id}-item`}>
      {/* ESP Item Image */}
      {item.image !== "" && (
        <div
          key={`${item.id}-${item.image}`}
          className="mt-4 aspect-video w-full overflow-hidden rounded-md border border-stone-300 p-0.5"
        >
          <div className="group relative h-full w-full overflow-hidden rounded-md">
            <img
              className="relative z-0 h-full w-full object-cover object-center"
              src={convertFileSrc(item.image)}
            />

            {/* Delete */}
            <button
              className="absolute right-2 top-2 z-10 hidden size-5 items-center justify-center rounded-full bg-amber-600 text-white transition-colors hover:bg-amber-500 active:bg-amber-600 group-hover:flex"
              onClick={() => {
                remove(item.image).then(() => {
                  onItemUpdated({ ...item, image: "" });
                });
              }}
            >
              <TrashIcon className="size-3" />
            </button>
          </div>
        </div>
      )}

      {/* ESP Item Input */}
      <div
        key={`item-input-${date}-${item.id}`}
        className="group relative flex h-14 w-full items-center "
      >
        <input
          key={`input-${date}-${item.id}`}
          type="text"
          className="peer relative z-0 block h-full w-full border-b border-b-stone-400 bg-app-white outline-none transition-colors focus:border-b-stone-700"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onBlur={() => {
            setValue(value.trim());
            onItemUpdated({ ...item, content: value.trim() });
          }}
        />

        {/* Choose Image */}
        {item.image === "" && (
          <ChooseImageButton
            onImageChosen={(file) => {
              saveImageToAppData({
                src: file,
                itemId: item.id,
                itemDate: date,
              }).then((path) => onItemUpdated({ ...item, image: path }));
            }}
          />
        )}
      </div>
    </div>
  );
}

// PRO Feature Component
function ChooseImageButton({
  onImageChosen,
}: {
  onImageChosen: (file: string) => void;
}) {
  // there's always a delay when opening File Dialog with Tauri
  // this lets the user know dialogOpen is in progress by showing Spinner
  const [isOpeningFile, setIsOpeningFile] = useState(false);
  const [_, setProModalOpened] = useAtom(proLicenseModalAtom);

  const chooseImage = async () => {
    setIsOpeningFile(true);
    
    const license = await getLicense();

    // check license and its entitlements
    if (
      license === null ||
      !license.valid ||
      !license.entitlements.includes("ADD_IMAGE")
    ) {
      setProModalOpened(true); // Show Modal: "This feature requires a PRO License"
      setIsOpeningFile(false);
      return;
    }

    const file = await openFileDialog({
      multiple: false,
      title: "Choose Image",
      filters: [
        {
          name: "Image",
          extensions: ["png", "webp", "avif", "jpg", "jpeg"],
        },
      ],
    });

    if (!Array.isArray(file) && file !== null) {
      onImageChosen(file);
    }

    setIsOpeningFile(false);
  };

  return (
    <button
      tabIndex={-1}
      className="group absolute right-0 z-10 hidden size-6 items-center justify-center rounded-full border border-stone-300 bg-app-white text-stone-400 transition-colors hover:border-stone-600 hover:text-stone-600 group-hover:flex peer-focus:flex"
      onClick={chooseImage}
    >
      {isOpeningFile ? <Spinner /> : <PhotoIcon className="size-3.5" />}
    </button>
  );
}

function Spinner() {
  return (
    <svg
      className="size-3.5 animate-spin text-current"
      xmlns="http://www.w3.org/2000/svg"
      fill="none"
      viewBox="0 0 24 24"
    >
      <circle
        className="opacity-25"
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="4"
      ></circle>
      <path
        className="opacity-75"
        fill="currentColor"
        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
      ></path>
    </svg>
  );
}
