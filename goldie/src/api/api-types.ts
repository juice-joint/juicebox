export enum Status {
  Success = "Success",
  InProgress = "InProgress",
  Failed = "Failed",
  Ready = "Ready",
}

export interface Song {
  name: string;
  uuid: string;
  status: Status;
}

export interface FormattedSong extends Song {
  formattedName: string;
}

export interface ServerIpResponse {
  ip: string;
}
