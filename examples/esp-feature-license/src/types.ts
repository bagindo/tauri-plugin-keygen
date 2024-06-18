export type ESPType = "effort" | "success" | "progress";

export type ESPItem = {
  id: string;
  type: ESPType;
  content: string;
  image: string;
};
