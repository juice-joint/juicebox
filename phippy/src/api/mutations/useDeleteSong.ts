import { useMutation } from "@tanstack/react-query";
import axios from "../axios";
import queryClient from "../queryClient";
import { QUERY_KEYS } from "../queryKeys";

export type DeleteSongParams = {
  song_uuid: string;
};

const deleteSong = async (song: DeleteSongParams) => {
  const response = await axios.post("/remove_song", song);
  return response;
};

export const useDeleteSong = () => {
  return useMutation({
    mutationFn: deleteSong,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.queue });
    },
  });
};
