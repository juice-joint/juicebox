import { Song } from "../api-types";

export enum EventType {
  QueueChange = "QueueUpdated",
  KeyChange = "KeyChange",
  TogglePlayback = "TogglePlayback",
}

export type QueueUpdatedEvent = {
  type: EventType.QueueChange;
  queue: Song[];
};

export type KeyChangeEvent = {
  type: EventType.KeyChange;
  current_key: number;
};

export type TogglePlaybackEvent = {
  type: EventType.TogglePlayback;
};

export type SSEEvent = QueueUpdatedEvent | TogglePlaybackEvent | KeyChangeEvent;
