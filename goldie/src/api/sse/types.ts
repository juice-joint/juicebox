import { FormattedSong } from "../api-types";

export enum EventType {
  QueueChangeEvent = "QueueUpdated",
  KeyChange = "KeyChange",
  TogglePlayback = "TogglePlayback",
  RestartSong = "RestartSong",
}

export type QueueUpdatedEvent = {
  type: EventType.QueueChangeEvent;
  queue: FormattedSong[];
};

export type KeyChangeEvent = {
  type: EventType.KeyChange;
  current_key: number;
};

export type TogglePlaybackEvent = {
  type: EventType.TogglePlayback;
};

export type RestartSongEvent = {
  type: EventType.RestartSong;
};

export type SSEEvent =
  | QueueUpdatedEvent
  | TogglePlaybackEvent
  | KeyChangeEvent
  | RestartSongEvent;
