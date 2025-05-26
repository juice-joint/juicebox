import QRCode from "react-qr-code";
import { useServerIp } from "../../api/queries/useServerIp";

function QRCodeBanner() {
  const { data: serverIp } = useServerIp();

  return (
    <div className="absolute bottom-4 left-4">
      <div className="bg-white p-3 rounded-lg shadow-xl flex flex-col items-center">
        <QRCode value={`http://${serverIp ?? ""}:8000/phippy`} size={128} />
        <p className="text-gray-800 text-sm mt-2 font-medium">Must be on WiFi!</p>
      </div>
    </div>
  );
}

export default QRCodeBanner;
