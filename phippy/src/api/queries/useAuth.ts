import { useQuery, useQueryClient } from "@tanstack/react-query";
import { QUERY_KEYS } from "../queryKeys";

const AUTH_KEY = "admin_authi";
const AUTH_EXPIRY = 60 * 60 * 1000; // 60 minutes in milliseconds

interface AuthState {
  isAuthenticated: boolean;
  timestamp: number;
}

const getStoredAuth = (): AuthState | null => {
  const storedAuth = localStorage.getItem(AUTH_KEY);
  if (!storedAuth) return null;

  const auth = JSON.parse(storedAuth);
  if (auth.isAuthenticated && Date.now() - auth.timestamp < AUTH_EXPIRY) {
    return auth;
  }

  localStorage.removeItem(AUTH_KEY);
  return null;
};

export const useAuth = () => {
  const queryClient = useQueryClient();

  const { data: auth } = useQuery<AuthState | null>({
    queryKey: QUERY_KEYS.auth,
    initialData: getStoredAuth,
    staleTime: Infinity,
  });

  const login = (password: string) => {
    if (password === import.meta.env.VITE_ADMIN_PASSWORD) {
      const newAuth = { isAuthenticated: true, timestamp: Date.now() };
      localStorage.setItem(AUTH_KEY, JSON.stringify(newAuth));
      queryClient.setQueryData(QUERY_KEYS.auth, newAuth);
      return true;
    }
    return false;
  };

  const logout = () => {
    localStorage.removeItem(AUTH_KEY);
    queryClient.setQueryData(QUERY_KEYS.auth, null);
  };

  return {
    isAuthenticated: auth?.isAuthenticated ?? false,
    login,
    logout,
  };
};
