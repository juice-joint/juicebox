import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Song } from "../api-types";
import { useQueue } from "../queries/useQueue";
import { QUERY_KEYS } from "../queryKeys";
import { SSE_URL } from "./eventSource";
import { EventType, type SSEEvent } from "./types";
import { formatSong } from "../../utils/format";

let eventSource: EventSource | null = null;

export const useEventSource = () => {
  const queryClient = useQueryClient();
  const getQueue = useQueue();

  // set up cleanup when query is removed from cache
  queryClient.setQueryDefaults(["sse", SSE_URL], {
    gcTime: 0,
    staleTime: Infinity,
    queryFn: () => {
      return new Promise((resolve) => {
        if (!eventSource) {
          eventSource = new EventSource(new URL(SSE_URL));
          getQueue.refetch();

          eventSource.onmessage = (event) => {
            try {
              const data = JSON.parse(event.data) as SSEEvent;
              switch (data.type) {
                case EventType.QueueChange:
                  queryClient.setQueryData<Song[]>(
                    QUERY_KEYS.queue,
                    data.queue.map(formatSong)
                  );
                  break;
                case EventType.KeyChange:
                  queryClient.setQueryData<number>(
                    QUERY_KEYS.key,
                    data.current_key
                  );
                  break;
                case EventType.TogglePlayback:
                  break;
                default:
                  console.error("invalid event type", data);
                  return;
              }
            } catch (e) {
              console.error("failed to parse sse data bestie:", e);
            }
          };

          eventSource.onerror = (err) => {
            console.error("sse connection error:", err);
            if (eventSource) {
              eventSource.close();
              eventSource = null;
            }
            // attempt to reconnect after a delay
            setTimeout(() => {
              queryClient.invalidateQueries({ queryKey: ["sse", SSE_URL] });
            }, 1000);
          };

          eventSource.onopen = () => {
            console.info("sse connection opened");
            resolve(null);
          };
        } else {
          resolve(null);
        }
      });
    },
  });

  const query = useQuery({
    queryKey: ["sse", SSE_URL],
    enabled: true,
  });

  if (eventSource?.readyState === EventSource.CLOSED) {
    eventSource = null;
    queryClient.invalidateQueries({ queryKey: ["sse", SSE_URL] });
  }

  return {
    ...query,
    isConnected: !query.isLoading,
  };
};
