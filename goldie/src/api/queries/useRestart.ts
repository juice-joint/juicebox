import { useQuery } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";

export const useRestart = () => {
  const { data: restart } = useQuery<boolean>({
    queryKey: QUERY_KEYS.restart,
    enabled: true,
    initialData: true,
  });

  return restart;
};
