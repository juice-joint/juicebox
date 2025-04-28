import { useMutation } from "@tanstack/react-query";
import axios from "../axios";

const restartSong = async () => {
  const response = await axios.post("/restart");
  return response;
};

export const useRestartSong = () => {
  return useMutation({
    mutationFn: restartSong,
  });
};
