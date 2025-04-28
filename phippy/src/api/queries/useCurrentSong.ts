import { useQueue } from "./useQueue";

export function useCurrentSong() {
  const { data: queue } = useQueue();
  return queue?.at(0);
}
