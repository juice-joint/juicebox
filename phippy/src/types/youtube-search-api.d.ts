declare module "youtube-search-api" {
  interface Thumbnail {
    url: string;
    width: number;
    height: number;
  }

  interface VideoItem {
    id: string;
    title: string;
    thumbnail: {
      thumbnails: Thumbnail[];
    };
    length?: {
      simpleText: string;
    };
  }

  interface SearchResponse {
    items: VideoItem[];
    estimatedResults: number;
    nextPageToken: string;
  }

  export const GetListByKeyword: (
    query: string,
    withPlaylist: boolean,
    limit: number
  ) => Promise<SearchResponse>;
}
