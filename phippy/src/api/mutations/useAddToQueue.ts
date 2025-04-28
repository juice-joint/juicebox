import { useMutation } from "@tanstack/react-query";
import axios from "../axios";
import queryClient from "../queryClient";
import { QUERY_KEYS } from "../queryKeys";

export type AddToQueueParams = {
  yt_link: string;
  name: string;
  is_key_changeable: boolean;
};

const addToQueue = async (song: AddToQueueParams) => {
  const response = await axios.post("/queue_song", song);
  console.log(song);

  return response;
};

export const useAddToQueue = () => {
  return useMutation({
    mutationFn: addToQueue,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.queue });
    },
  });
};
