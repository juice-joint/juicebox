import { useMutation } from "@tanstack/react-query";
import axios from "../axios";
import queryClient from "../queryClient";
import { QUERY_KEYS } from "../queryKeys";
import { FormattedSong } from "../api-types";

export type RepositionParams = {
  position: number;
  song_uuid: string;
};

const reposition = async (repositionData: RepositionParams) => {
  const response = await axios.post("/reposition_song", repositionData);
  return response;
};

export const useReposition = () => {
  return useMutation({
    mutationFn: reposition,
    onMutate(variables) {
      const previousQueue = queryClient.getQueryData<FormattedSong[]>(
        QUERY_KEYS.queue
      );
      if (!previousQueue) return;

      const newQueue = [...previousQueue];
      const songIndex = newQueue.findIndex(
        (song) => song.uuid === variables.song_uuid
      );
      if (songIndex === -1) return;

      const [song] = newQueue.splice(songIndex, 1);
      newQueue.splice(variables.position, 0, song);

      queryClient.setQueryData(QUERY_KEYS.queue, newQueue);
      return { previousQueue };
    },
    // onSuccess: () => {
    //   queryClient.invalidateQueries({ queryKey: QUERY_KEYS.queue });
    // },
  });
};
