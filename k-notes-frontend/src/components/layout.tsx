import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from "@/components/app-sidebar"
import { Outlet } from "react-router-dom"
import { Button } from "@/components/ui/button";
import { LogOut } from "lucide-react";
import { useLogout, useUser } from "@/hooks/use-auth";
import { ModeToggle } from "@/components/mode-toggle";
import { BulkSelectionProvider } from "@/components/bulk-selection-context";
import { BulkActionsBar } from "@/components/bulk-actions-bar";

export default function Layout() {
  const { mutate: logout } = useLogout();
  const { data: user } = useUser();

  return (
    <BulkSelectionProvider>
      <SidebarProvider>
        <AppSidebar />
        <main className="w-full flex flex-col min-h-screen">
          <header className="border-b bg-background/95 backdrop-blur h-14 flex items-center justify-between px-4 sticky top-0 z-10">
            <div className="flex items-center gap-2">
              <SidebarTrigger />
              <img src="/logo.png" alt="K-Notes Logo" className="h-8 w-8 object-contain" />
              <div className="font-semibold">K-Notes</div>
            </div>

            <div className="flex items-center gap-2">
              <div className="text-sm text-muted-foreground hidden sm:block">
                {user?.email}
              </div>
              <ModeToggle />
              <Button variant="ghost" size="icon" onClick={() => logout()} title="Logout">
                <LogOut className="h-4 w-4" />
              </Button>
            </div>
          </header>
          <div className="flex-1 p-4 md:p-6 bg-muted/10">
            <Outlet />
          </div>
        </main>
        <BulkActionsBar />
      </SidebarProvider>
    </BulkSelectionProvider>
  )
}

