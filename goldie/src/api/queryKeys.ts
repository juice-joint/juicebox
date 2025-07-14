import { EventType } from "./sse/types";

export const QUERY_KEYS = {
  playNextSong: ["playNextSong"] as const,
  ip: ["ip"] as const,
  key: ["key"] as const,
  playback: ["playback"] as const,
  restart: ["restart"] as const,
  queue: ["sse", EventType.QueueChangeEvent] as const,
  autoApStatus: ["autoApStatus"] as const,
};
