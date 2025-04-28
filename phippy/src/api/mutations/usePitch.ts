import { useMutation } from "@tanstack/react-query";
import axios from "../axios";

export type AddToQueueParams = {
  yt_link: string;
  name: string;
};

const keyUp = async () => {
  const response = await axios.post("/key_up");
  return response;
};

const keyDown = async () => {
  const response = await axios.post("/key_down");
  return response;
};

export const useKeyUp = () => {
  return useMutation({
    mutationFn: keyUp,
  });
};

export const useKeyDown = () => {
  return useMutation({
    mutationFn: keyDown,
  });
};
