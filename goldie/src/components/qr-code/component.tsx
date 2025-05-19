import { useServerIp } from "../../api/queries/useServerIp";

function QRCodeBanner() {
  const { data: serverIp } = useServerIp();
  console.log(serverIp)

  return (
    <div className="absolute bottom-4 left-4">
      <div className="bg-white p-3 rounded-lg shadow-xl flex flex-col items-center">
        {/* <QRCode value={`http://${serverIp ?? ""}:8000`} size={128} /> */}
      </div>
    </div>
  );
}

export default QRCodeBanner;
