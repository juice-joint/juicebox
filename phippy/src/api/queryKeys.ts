import { EventType } from "./sse/types";

export const QUERY_KEYS = {
  playNextSong: ["playNextSong"] as const,
  queue: ["sse", EventType.QueueChange] as const,
  key: ["key"] as const,
  auth: ["auth"] as const,
};
