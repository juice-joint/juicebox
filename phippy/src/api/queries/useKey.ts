import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";
import axiosClient from "../axios";

const fetchKey = async () => {
  const response = await axiosClient.get<number>("get_key");

  return response.data;
};

export const useKey = () => {
  const { data: key } = useQuery<number>({
    queryFn: fetchKey,
    queryKey: QUERY_KEYS.key,
    enabled: true,
  });

  return key;
};
