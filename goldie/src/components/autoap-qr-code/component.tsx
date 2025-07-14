import QRCode from "react-qr-code";
import { useServerIp } from "../../api/queries/useServerIp";
import { useAutoApStatus } from "../../api/queries/useAutoApStatus";

function AutoApQRCode() {
  const { data: serverIp } = useServerIp();
  const { data: autoApStatus } = useAutoApStatus();

  // Only show QR code if autoap is running
  if (!autoApStatus?.is_running) {
    return null;
  }

  // Use the detected port or fallback to 8080
  const port = autoApStatus.web_server_port || 8080;

  return (
    <div className="absolute bottom-4 left-4">
      <div className="bg-white p-3 rounded-lg shadow-xl flex flex-col items-center">
        <QRCode value={`http://${serverIp ?? ""}:${port}`} size={128} />
        <p className="text-gray-800 text-sm mt-2 font-medium">WiFi Setup</p>
        <p className="text-gray-600 text-xs">Connect to configure</p>
      </div>
    </div>
  );
}

export default AutoApQRCode;