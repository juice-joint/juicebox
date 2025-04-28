export enum Status {
  Success = "Success",
  InProgress = "InProgress",
  Failed = "Failed",
}

export interface Song {
  name: string;
  uuid: string;
  status: Status;
  is_key_changeable: boolean;
}

export interface FormattedSong extends Song {
  formattedName: string;
}

export interface ServerIpResponse {
  ip: string;
}
