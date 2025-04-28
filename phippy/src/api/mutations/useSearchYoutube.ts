import { useQuery } from "@tanstack/react-query";
import axios from "../axios";

export type SearchResult = {
  title: string;
  url: string;
  id: string;
};

export type SearchSongParams = {
  query: string;
};

const searchYoutube = async (query: string) => {
  const response = await axios.get<Array<SearchResult>>("/search", {
    params: {
      query: `${query} karaoke`,
    },
  });
  return response.data;
};

export const useSearchYoutube = (query: string) => {
  return useQuery({
    queryKey: ["youtube-search", query],
    queryFn: () => searchYoutube(query),
    enabled: !!query.trim(),
  });
};
