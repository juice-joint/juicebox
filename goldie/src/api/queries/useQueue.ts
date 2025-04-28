import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";
import { Song } from "../api-types";
import { formatSong } from "../../utils/format";

async function getQueue() {
  const { data } = await axiosClient.get<Array<Song>>("/song_list", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });

  const formattedQueue = data.map(formatSong);
  return formattedQueue;
}

export function useQueue() {
  return useQuery({
    queryFn: getQueue,
    queryKey: QUERY_KEYS.queue,
  });
}
