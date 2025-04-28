import { useQuery } from "@tanstack/react-query";
import { formatSong } from "../../utils/format";
import { FormattedSong, Song } from "../api-types";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";

async function getQueue(): Promise<Array<FormattedSong>> {
  const { data } = await axiosClient.get<Array<Song>>("/song_list");
  return data.map(formatSong);
}

export function useQueue() {
  return useQuery<Array<FormattedSong>>({
    queryFn: getQueue,
    queryKey: QUERY_KEYS.queue,
  });
}
