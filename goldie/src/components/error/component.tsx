import React, { useEffect, useState } from "react";
import rubbo from "../../assets/images/rubbo.jpg";

interface ErrorProps {
  message?: string;
}

export const ErrorScreen: React.FC<ErrorProps> = ({
  message = "oops! something went wrong",
}) => {
  const [timer, setTimer] = useState(5);
  useEffect(() => {
    const interval = setInterval(() => {
      setTimer((prev) => prev - 1);
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  if (timer === 0) {
    window.location.reload();
  }

  return (
    <div className="flex flex-col items-center justify-center w-full h-full bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900 text-center">
      <div className="animate-bounce-slow flex flex-row items-center mb-4">
        <img
          src={rubbo}
          alt="Splash screen"
          className="max-h-[200px] h-auto rounded-2xl"
        />
      </div>
      <h2 className="text-2xl text-white font-medium mb-6">{message}</h2>
      <p className="text-white text-lg">refreshing in {timer} seconds</p>
    </div>
  );
};
