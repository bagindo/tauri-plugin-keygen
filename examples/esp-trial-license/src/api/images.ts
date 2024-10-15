import { appDataDir, BaseDirectory, extname, join } from "@tauri-apps/api/path";
import { copyFile, mkdir, exists } from "@tauri-apps/plugin-fs";

export async function saveImageToAppData({
  src,
  itemId,
  itemDate,
}: {
  src: string;
  itemId: string;
  itemDate: string;
}) {
  try {
    // make sure src still exists
    const srcExists = await exists(src);

    if (!srcExists) {
      console.error(
        `failed saving image to app data: src path ${src} doesn't exist anymore`,
      );
      return "";
    }

    // set image name
    const ext = await extname(src);
    const imageName = `${itemDate}-${itemId}-${Date.now()}${ext === "" ? "" : `.${ext}`}`;

    // get images dir
    const imagesDirPath = await getImagesDirPath();

    // get dest path
    const dest = await join(imagesDirPath, imageName);

    // copy file
    await copyFile(src, dest);

    return dest;
  } catch (e) {
    console.error("failed saving image to app data");
    console.error(e);
    return "";
  }
}

async function getImagesDirPath() {
  // app data path
  const espAppData = await appDataDir();

  // images dir
  const imagesDirPath = await join(espAppData, "images");

  // create if hasn't exist
  const imagesDirExists = await exists(imagesDirPath);
  if (!imagesDirExists) {
    await mkdir(imagesDirPath, { recursive: true, baseDir: BaseDirectory.AppLocalData });
  }

  return imagesDirPath;
}
