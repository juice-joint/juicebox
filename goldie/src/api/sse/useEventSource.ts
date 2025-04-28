import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect } from "react";
import { formatSong } from "../../utils/format";
import { Song } from "../api-types";
import { useQueue } from "../queries/useQueue";
import { QUERY_KEYS } from "../queryKeys";
import { SSE_URL } from "./eventSource";
import { EventType, type SSEEvent } from "./types";

const sseConnections = new Map<string, EventSource>();

export const useEventSource = () => {
  const queryClient = useQueryClient();
  const getQueue = useQueue();

  const query = useQuery({
    gcTime: 0,
    staleTime: Infinity,
    queryKey: ["sse", SSE_URL],
    queryFn: () => {
      return new Promise((resolve) => {
        if (!sseConnections.has(SSE_URL)) {
          const eventSource = new EventSource(new URL(SSE_URL));
          sseConnections.set(SSE_URL, eventSource);
          getQueue.refetch();

          eventSource.onmessage = (event) => {
            try {
              const data = JSON.parse(event.data) as SSEEvent;
              switch (data.type) {
                case EventType.QueueChangeEvent:
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
                case EventType.TogglePlayback: {
                  const oldQueryData = queryClient.getQueryData<boolean>(
                    QUERY_KEYS.playback
                  );
                  queryClient.setQueryData(QUERY_KEYS.playback, !oldQueryData);
                  break;
                }
                case EventType.RestartSong: {
                  const oldQueryData = queryClient.getQueryData<boolean>(
                    QUERY_KEYS.restart
                  );
                  queryClient.setQueryData(QUERY_KEYS.restart, !oldQueryData);
                  break;
                }
                default:
                  console.warn("invalid event type", data);
                  return;
              }
            } catch (e) {
              console.error("failed to parse sse data bestie:", e);
            }
          };

          eventSource.onerror = () => {
            sseConnections.delete(SSE_URL);
            eventSource.close();
          };

          // resolve once connection is open
          eventSource.onopen = () => resolve(null);
        } else {
          // if connection exists, resolve immediately
          resolve(null);
        }
      });
    },
    enabled: true,
  });

  // cleanup on unmount
  useEffect(() => {
    return () => {
      if (
        queryClient
          .getQueryCache()
          .find({ queryKey: ["sse", SSE_URL] })
          ?.getObserversCount() === 1
      ) {
        sseConnections.get(SSE_URL)?.close();
        sseConnections.delete(SSE_URL);
      }
    };
  }, [queryClient]);

  return {
    ...query,
    isConnected: !query.isLoading,
  };
};
