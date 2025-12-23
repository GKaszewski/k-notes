import { useUser } from "@/hooks/use-auth";
import { Navigate, Outlet } from "react-router-dom";

export function ProtectedRoute() {
  const { data: user, isLoading } = useUser();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full" />
      </div>
    );
  }

  if (!user) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}

export function PublicRoute() {
  const { data: user, isLoading } = useUser();

  if (isLoading) {
    return null; // Or loader
  }

  if (user) {
    return <Navigate to="/" replace />;
  }

  return <Outlet />;
}
