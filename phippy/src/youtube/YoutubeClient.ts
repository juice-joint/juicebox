import axios from "axios";

export interface YoutubeSearchResponse {
  kind: string;
  etag: string;
  nextPageToken: string;
  regionCode: string;
  pageInfo: PageInfo;
  items: Item[];
}

export interface Item {
  kind: ItemKind;
  etag: string;
  id: ID;
  snippet: Snippet;
}

export interface ID {
  kind: IDKind;
  videoId: string;
}

export enum IDKind {
  YoutubeVideo = "youtube#video",
}

export enum ItemKind {
  YoutubeSearchResult = "youtube#searchResult",
}

export interface Snippet {
  publishedAt: Date;
  channelId: string;
  title: string;
  description: string;
  thumbnails: Thumbnails;
  channelTitle: string;
  liveBroadcastContent: LiveBroadcastContent;
  publishTime: Date;
}

export enum LiveBroadcastContent {
  None = "none",
}

export interface Thumbnails {
  default: Default;
  medium: Default;
  high: Default;
}

export interface Default {
  url: string;
  width: number;
  height: number;
}

export interface PageInfo {
  totalResults: number;
  resultsPerPage: number;
}

export class YoutubeClient {
  private readonly apiKey: string;
  private readonly baseUrl = "https://youtube.googleapis.com/youtube/v3";

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async search(query: string, maxResults = 10) {
    const response = await axios.get<YoutubeSearchResponse>(
      `${this.baseUrl}/search`,
      {
        params: {
          part: "snippet",
          type: "video",
          maxResults,
          q: query + "karaoke",
          key: this.apiKey,
        },
      }
    );

    return response.data;
  }
}

const youtubeClient = new YoutubeClient(import.meta.env.VITE_APP_YT_API_KEY);
export { youtubeClient };
