import { Song } from "../api/api-types";

export const formatSong = (song: Song) => {
  const rawName = song.name;
  const formattedName = rawName.replace("./assets/", "").replaceAll("_", " ");
  return { ...song, formattedName };
};
