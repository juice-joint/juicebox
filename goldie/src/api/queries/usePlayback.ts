import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";

export const usePlayback = () => {
  const { data: playbackState } = useQuery<boolean>({
    queryKey: QUERY_KEYS.playback,
    enabled: true,
    initialData: true,
  });

  return playbackState;
};
