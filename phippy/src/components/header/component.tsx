import { MdAdminPanelSettings } from "react-icons/md";

interface HeaderProps {
  onOpenAdmin: () => void;
}

export const Header = ({ onOpenAdmin }: HeaderProps) => {
  return (
    <header className="sticky top-0 z-20 w-full h-14 flex justify-between items-center bg-gradient-to-br from-purple-900 via-indigo-900 to-blue-900 backdrop-blur-sm px-4">
      <h1 className="text-xl font-bold text-white">Pi-tch Perfect</h1>
      <button
        onClick={onOpenAdmin}
        className="p-2 active:bg-white/10 rounded-lg transition-colors text-white"
      >
        <MdAdminPanelSettings />
      </button>
    </header>
  );
};
