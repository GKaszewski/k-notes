import { Routes, Route, Navigate } from "react-router-dom";
import { ProtectedRoute, PublicRoute } from "@/components/auth-guard";
import LoginPage from "@/pages/login";
import RegisterPage from "@/pages/register";
import DashboardPage from "@/pages/dashboard";
import Layout from "@/components/layout";

function App() {
  return (
    <Routes>
      {/* Public Routes (only accessible if NOT logged in) */}
      <Route element={<PublicRoute />}>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
      </Route>

      {/* Protected Routes (only accessible if logged in) */}
      <Route element={<ProtectedRoute />}>
        <Route element={<Layout />}>
            <Route path="/" element={<DashboardPage />} />
            <Route path="/archive" element={<DashboardPage />} />
        </Route>
      </Route>

      {/* Catch all redirect */}
      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default App;
