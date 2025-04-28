import { useQuery } from "@tanstack/react-query";
import axiosClient from "../axios";
import { QUERY_KEYS } from "../queryKeys";
import { ServerIpResponse } from "../api-types";

async function getServerIp() {
  const { data } = await axiosClient.get<ServerIpResponse>("/server_ip", {
    headers: { "Content-Type": "application/json", Accept: "*" },
  });

  return data.ip;
}

export function useServerIp() {
  return useQuery({
    queryFn: getServerIp,
    queryKey: QUERY_KEYS.ip,
  });
}
