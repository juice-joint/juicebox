import React from "react";
import rubbo from "../../assets/images/rubbo.jpg";

interface SplashProps {
  message?: string;
}

export const Splash: React.FC<SplashProps> = ({
  message = "waiting for next song...",
}) => {
  return (
    <div className="flex flex-col items-center justify-center w-full h-full bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900 text-center">
      <div className="animate-bounce-slow flex flex-row items-center mb-4">
        <img
          src={rubbo}
          alt="Splash screen"
          className="max-h-[200px] h-auto rounded-2xl"
        />
        <h1 className="text-8xl text-white font-bold p-4 leading-loose mb-4">
          pi-tch perfect
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
