import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";
import type { QueueUpdatedEvent } from "./types";

export const useQueueChanges = () => {
  const { data: queueData } = useQuery<QueueUpdatedEvent["queue"]>({
    queryKey: QUERY_KEYS.queue,
    enabled: true,
  });

  return queueData || [];
};
