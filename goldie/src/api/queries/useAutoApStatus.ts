import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";
import { AutoApStatusResponse } from "../api-types";

async function getAutoApStatus() {
  const { data } = await axiosClient.get<AutoApStatusResponse>("/autoap_status", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });

  return data;
}

export function useAutoApStatus() {
  return useQuery({
    queryFn: getAutoApStatus,
    queryKey: QUERY_KEYS.autoApStatus,
    refetchInterval: 5000,
  });
}