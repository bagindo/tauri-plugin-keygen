import { Store } from "@tauri-apps/plugin-store"; // should use a db instead for prod app
import type { ESPItem } from "../types";

const STORE_PATH = ".esp.bin";

export async function getESPItems({ date }: { date: string }) {
  try {
    // load store
    const espStore = await Store.load(STORE_PATH);

    // get esp items for "YYYY-MM-DD"
    const dateItems = await espStore.get<ESPItem[]>(date);

    return dateItems === undefined ? [] : dateItems;
  } catch (e) {
    console.error(e);
    return [];
  }
}

export async function upsertESPItem({
  date,
  espItem,
}: {
  date: string;
  espItem: ESPItem;
}) {
  try {
    // load store
    const espStore = await Store.load(STORE_PATH);

    // get esp items for "YYYY-MM-DD"
    const dateItems = await espStore.get<ESPItem[]>(date);

    // no items yet for "YYYY-MM-DD"
    if (dateItems === undefined) {
      await espStore.set(date, [espItem]); // INSERT
      await espStore.save();
    }
    // has items
    else {
      // check if espItem already exists
      const oldItem = dateItems.find((item) => item.id === espItem.id);

      // espItem exists -> UPDATE
      if (oldItem !== undefined) {
        await espStore.set(
          date,
          dateItems.map((item) => (item.id === espItem.id ? espItem : item)),
        );
        await espStore.save();
      }
      // espItem doesn't exist -> INSERT
      else {
        await espStore.set(date, [...dateItems, espItem]);
        await espStore.save();
      }
    }
  } catch (e) {
    console.error(e);
  }
}

export async function deleteESPItem({
  id,
  date,
}: {
  id: string;
  date: string;
}) {
  try {
    const espStore = await Store.load(STORE_PATH);

    // get esp items for "YYYY-MM-DD"
    const dateItems = await espStore.get<ESPItem[]>(date);

    if (dateItems !== undefined) {
      // get old item
      const oldItem = dateItems.find((item) => item.id === id);

      if (oldItem !== undefined) {
        await espStore.set(
          date,
          dateItems.filter((item) => item.id !== id),
        );
        await espStore.save();
      }
    }
  } catch (e) {
    console.error(e);
  }
}
