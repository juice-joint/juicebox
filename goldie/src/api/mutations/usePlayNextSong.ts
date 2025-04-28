import { useMutation, useQueryClient } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";
import axiosClient from "../axios";

async function playNextSong() {
  const data = await axiosClient.post("/play_next", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });
  return data;
}

export function usePlayNextSong() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: playNextSong,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: QUERY_KEYS.key });
    },
  });
}
