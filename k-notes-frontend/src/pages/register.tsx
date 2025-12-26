import { useState, useEffect } from "react";
import { useForm } from "react-hook-form";
import { Settings } from "lucide-react";
import { SettingsDialog } from "@/components/settings-dialog";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import { Link, useNavigate } from "react-router-dom";
import { useRegister } from "@/hooks/use-auth";
import { useConfig } from "@/hooks/useConfig";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from "@/components/ui/form";
import { ApiError } from "@/lib/api";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";

const registerSchema = z.object({
  email: z.string().email("Invalid email address"),
  password: z.string().min(6, "Password must be at least 6 characters"),
  confirmPassword: z.string().min(6, "Password must be at least 6 characters"),
}).refine((data) => data.password === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"],
});

type RegisterFormValues = z.infer<typeof registerSchema>;

export default function RegisterPage() {
  const { mutate: register, isPending } = useRegister();
  const { data: config, isLoading: isConfigLoading } = useConfig();
  const navigate = useNavigate();
  const { t } = useTranslation();

  useEffect(() => {
    if (!isConfigLoading && config?.allow_registration === false) {
      toast.error(t("Registration is currently disabled"));
      navigate("/login");
    }
  }, [config, isConfigLoading, navigate, t]);

  if (isConfigLoading || config?.allow_registration === false) {
    return null; // Or a loading spinner
  }

  const form = useForm<RegisterFormValues>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      email: "",
      password: "",
      confirmPassword: "",
    },
  });

  const onSubmit = (data: RegisterFormValues) => {
    register({
      email: data.email,
      password: data.password,
    }, {
      onError: (error: any) => {
        if (error instanceof ApiError) {
          toast.error(error.message);
        } else {
          toast.error(t("Failed to register"));
        }
      },
    });
  };

  const [settingsOpen, setSettingsOpen] = useState(false);

  return (
    <div className="flex min-h-screen items-center justify-center bg-gray-50 dark:bg-gray-950 p-4 relative">
      <div className="absolute top-4 right-4">
        <Button variant="ghost" size="icon" onClick={() => setSettingsOpen(true)}>
          <Settings className="h-5 w-5" />
        </Button>
      </div>
      <Card className="w-full max-w-md">
        <CardHeader>
          <CardTitle className="text-2xl font-bold">{t("Create an account")}</CardTitle>
          <CardDescription>
            {t("Enter your email below to create your account")}
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              <FormField
                control={form.control}
                name="email"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>{t("Email")}</FormLabel>
                    <FormControl>
                      <Input placeholder="name@example.com" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="password"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>{t("Password")}</FormLabel>
                    <FormControl>
                      <Input type="password" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="confirmPassword"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>{t("Confirm Password")}</FormLabel>
                    <FormControl>
                      <Input type="password" {...field} />
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Button type="submit" className="w-full" disabled={isPending}>
                {isPending ? t("Creating account...") : t("Create account")}
              </Button>
            </form>
          </Form>
        </CardContent>
        <CardFooter className="flex justify-center">
          <p className="text-sm text-gray-500 dark:text-gray-400">
            {t("Already have an account?")}{" "}
            <Link to="/login" className="font-semibold text-primary hover:underline">
              {t("Sign in")}
            </Link>
          </p>
        </CardFooter>
      </Card>
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} dataManagementEnabled={false} />
    </div>
  );
}
