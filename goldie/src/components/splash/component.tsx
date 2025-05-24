import React from "react";
import rubbo from "../../assets/images/output-onlinepngtools(2).png";

interface SplashProps {
  message?: string;
}

export const Splash: React.FC<SplashProps> = ({
  message = "waiting for next song...",
}) => {
  return (
    <div className="flex flex-col items-center justify-center w-full h-full bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900 text-center">
      <div className="animate-bounce-slow flex flex-row items-end mb-16">
        <img
          src={rubbo}
          alt="Splash screen"
          className="w-52 h-auto rounded-2xl pr-4"
        />
        <h1 className="text-8xl text-white font-bold p-4">
          juicebox
        </h1>
      </div>
      <h2 className="text-4xl text-white font-bold mb-4 animate-pulse">
        {message}
      </h2>
      <p className="text-purple-200 text-lg animate-fade-in">
        queue songs from your phone
      </p>
    </div>
  );
};
