import { useMutation } from "@tanstack/react-query";
import axios from "../axios";
import queryClient from "../queryClient";
import { QUERY_KEYS } from "../queryKeys";

const skip = async () => {
  const response = await axios.post("/play_next");
  return response;
};

const togglePlayback = async () => {
  const response = await axios.post("/toggle_playback");
  return response;
};

export const useSkip = () => {
  return useMutation({
    mutationFn: skip,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.key });
    },
  });
};

export const useTogglePlayback = () => {
  return useMutation({
    mutationFn: togglePlayback,
  });
};
