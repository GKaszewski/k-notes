import { FileText, Shield, Database, Lock, Mail, Calendar } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function PrivacyPolicyPage() {
    const lastUpdated = "December 26, 2025";
    const appName = "K-Notes";

    return (
        <div className="min-h-screen bg-linear-to-br from-background via-background to-muted/20">
            <div className="max-w-4xl mx-auto px-4 py-12 space-y-8">
                {/* Header */}
                <div className="text-center space-y-4 mb-12">
                    <div className="flex justify-center">
                        <Shield className="h-16 w-16 text-primary" />
                    </div>
                    <h1 className="text-4xl font-bold bg-clip-text text-transparent bg-linear-gradient-to-r from-primary to-primary/60">
                        Privacy Policy
                    </h1>
                    <div className="flex items-center justify-center gap-2 text-muted-foreground">
                        <Calendar className="h-4 w-4" />
                        <p className="text-sm">Last Updated: {lastUpdated}</p>
                    </div>
                </div>

                {/* Introduction */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <FileText className="h-5 w-5" />
                            Introduction
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <div className="bg-primary/10 border border-primary/20 rounded-lg p-4 mb-4">
                            <p className="font-semibold text-foreground mb-2">🏠 Self-Hosted Application</p>
                            <p className="text-sm">
                                {appName} is designed as a self-hosted application. This means you run your own
                                instance of the backend server, and <strong className="text-foreground">you have complete
                                    control over your data</strong>. The app developer does not collect, store, or have
                                access to any of your personal information or notes.
                            </p>
                        </div>
                        <p>
                            This Privacy Policy describes how the {appName} application handles data when you
                            self-host it. Since you control the backend infrastructure, you are responsible for
                            the security and privacy of your own data.
                        </p>
                        <p>
                            Please read this privacy policy carefully to understand how the application processes
                            information on your self-hosted instance.
                        </p>
                    </CardContent>
                </Card>

                {/* Information We Collect */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Database className="h-5 w-5" />
                            Data Stored on Your Instance
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p className="text-sm bg-muted/50 p-3 rounded-md">
                            <strong className="text-foreground">Important:</strong> All data described below is stored
                            exclusively on your self-hosted backend server. The app developer has no access to this data.
                        </p>

                        <div>
                            <h3 className="font-semibold text-foreground mb-2">Account Information</h3>
                            <p>Your self-hosted instance stores:</p>
                            <ul className="list-disc list-inside ml-4 mt-2 space-y-1">
                                <li>Email address (for account authentication on your instance)</li>
                                <li>Username</li>
                                <li>Password (hashed using industry-standard encryption)</li>
                            </ul>
                        </div>

                        <div>
                            <h3 className="font-semibold text-foreground mb-2">User Content</h3>
                            <p>Your instance stores the content you create:</p>
                            <ul className="list-disc list-inside ml-4 mt-2 space-y-1">
                                <li>Notes and their content</li>
                                <li>Tags and categories</li>
                                <li>Metadata (creation date, modification date, etc.)</li>
                            </ul>
                        </div>

                        <div>
                            <h3 className="font-semibold text-foreground mb-2">Technical Data</h3>
                            <p>Your instance may log technical information:</p>
                            <ul className="list-disc list-inside ml-4 mt-2 space-y-1">
                                <li>Server logs (if you enable logging)</li>
                                <li>Session data for authentication</li>
                                <li>Any other data you configure your instance to collect</li>
                            </ul>
                        </div>
                    </CardContent>
                </Card>

                {/* How We Use Your Information */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Lock className="h-5 w-5" />
                            How the Application Uses Data
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>The {appName} application uses data stored on your instance for:</p>
                        <ul className="list-disc list-inside ml-4 space-y-2">
                            <li><strong className="text-foreground">Core Functionality:</strong> To provide note-taking features, organize content, and manage your account</li>
                            <li><strong className="text-foreground">Synchronization:</strong> To sync your notes across devices when using the same instance</li>
                            <li><strong className="text-foreground">Authentication:</strong> To secure your account and protect your data from unauthorized access</li>
                            <li><strong className="text-foreground">Data Integrity:</strong> To maintain the consistency and reliability of your notes</li>
                        </ul>
                        <p className="mt-4 text-sm bg-muted/50 p-3 rounded-md">
                            Since you control the backend, you decide how your data is used, stored, and managed.
                            The app developer does not have access to or control over your self-hosted instance.
                        </p>
                    </CardContent>
                </Card>

                {/* Data Storage and Security */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Shield className="h-5 w-5" />
                            Data Storage and Security
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p className="text-sm bg-amber-500/10 border border-amber-500/20 p-3 rounded-md">
                            <strong className="text-foreground">Your Responsibility:</strong> As a self-hosted application,
                            the security of your data depends on how you configure and maintain your instance. You are
                            responsible for securing your server infrastructure.
                        </p>

                        <p className="mt-4">
                            The {appName} application includes the following security features:
                        </p>
                        <ul className="list-disc list-inside ml-4 space-y-2">
                            <li><strong className="text-foreground">Password Hashing:</strong> Passwords are hashed using industry-standard algorithms (never stored in plain text)</li>
                            <li><strong className="text-foreground">Session Management:</strong> Secure session handling for authenticated users</li>
                            <li><strong className="text-foreground">HTTPS Support:</strong> The application supports HTTPS when properly configured on your server</li>
                        </ul>

                        <div className="mt-4 p-3 bg-muted/50 rounded-md space-y-2">
                            <p className="font-semibold text-foreground">Security Recommendations:</p>
                            <ul className="list-disc list-inside ml-4 space-y-1 text-sm">
                                <li>Always use HTTPS in production</li>
                                <li>Keep your server software and dependencies updated</li>
                                <li>Use strong passwords and enable proper authentication</li>
                                <li>Regularly backup your data</li>
                                <li>Follow security best practices for your hosting environment</li>
                            </ul>
                        </div>
                    </CardContent>
                </Card>

                {/* Data Retention */}
                <Card>
                    <CardHeader>
                        <CardTitle>Data Retention and Deletion</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>
                            Since you control your own {appName} instance, you have complete control over data retention:
                        </p>
                        <ul className="list-disc list-inside ml-4 space-y-2">
                            <li>You can delete your account and all associated data at any time through the application</li>
                            <li>You can export your data using the built-in export functionality</li>
                            <li>You control backup and archival policies for your instance</li>
                            <li>You can permanently delete all data by removing your instance's database</li>
                        </ul>
                        <p className="mt-4">
                            The app developer does not retain any of your data, as all information exists solely
                            on your self-hosted server.
                        </p>
                    </CardContent>
                </Card>

                {/* Third-Party Services */}
                <Card>
                    <CardHeader>
                        <CardTitle>Third-Party Services and Data Sharing</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>
                            <strong className="text-foreground">No Data Sharing by Developer:</strong> The app developer
                            does not share your data with any third parties, as they do not have access to it.
                        </p>
                        <p>
                            However, if you integrate third-party services with your self-hosted instance (such as
                            external authentication providers, backup services, or hosting platforms), those services
                            may have access to your data according to their own privacy policies.
                        </p>
                        <p className="text-sm bg-muted/50 p-3 rounded-md">
                            Review the privacy policies of any third-party services you choose to integrate with
                            your instance.
                        </p>
                    </CardContent>
                </Card>

                {/* Children's Privacy */}
                <Card>
                    <CardHeader>
                        <CardTitle>Children's Privacy</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>
                            Our service is not intended for use by children under the age of 13. We do not knowingly
                            collect personally identifiable information from children under 13. If you are a parent
                            or guardian and you are aware that your child has provided us with personal information,
                            please contact us so we can take necessary action.
                        </p>
                    </CardContent>
                </Card>

                {/* Your Data Rights */}
                <Card>
                    <CardHeader>
                        <CardTitle>Your Data Rights</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>Depending on your location, you may have the following rights regarding your personal data:</p>
                        <ul className="list-disc list-inside ml-4 space-y-2">
                            <li><strong className="text-foreground">Access:</strong> Request access to your personal data</li>
                            <li><strong className="text-foreground">Correction:</strong> Request correction of inaccurate data</li>
                            <li><strong className="text-foreground">Deletion:</strong> Request deletion of your personal data</li>
                            <li><strong className="text-foreground">Export:</strong> Request a copy of your data in a portable format</li>
                            <li><strong className="text-foreground">Objection:</strong> Object to processing of your personal data</li>
                        </ul>
                        <p className="mt-4">
                            You can exercise many of these rights directly through the app's settings page
                            (export/import data functionality). For other requests, please contact us.
                        </p>
                    </CardContent>
                </Card>

                {/* Changes to This Policy */}
                <Card>
                    <CardHeader>
                        <CardTitle>Changes to This Privacy Policy</CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>
                            We may update our Privacy Policy from time to time. We will notify you of any changes
                            by posting the new Privacy Policy on this page and updating the "Last Updated" date.
                        </p>
                        <p>
                            You are advised to review this Privacy Policy periodically for any changes. Changes
                            to this Privacy Policy are effective when they are posted on this page.
                        </p>
                    </CardContent>
                </Card>

                {/* Contact Us */}
                <Card>
                    <CardHeader>
                        <CardTitle className="flex items-center gap-2">
                            <Mail className="h-5 w-5" />
                            Questions or Concerns
                        </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-4 text-muted-foreground">
                        <p>
                            If you have questions about this Privacy Policy or how {appName} handles data,
                            you can:
                        </p>
                        <ul className="list-disc list-inside ml-4 space-y-2">
                            <li>Review the source code and documentation on GitHub</li>
                            <li>Open an issue in the project repository</li>
                            <li>Contact the project maintainer</li>
                        </ul>
                        <div className="bg-muted/50 rounded-lg p-4 mt-4">
                            <p className="text-sm">
                                <strong className="text-foreground">Remember:</strong> As the operator of your own
                                instance, you control your data. For questions about data stored on your server,
                                you are responsible for your own data management practices.
                            </p>
                        </div>
                    </CardContent>
                </Card>

                {/* Footer */}
                <div className="text-center text-sm text-muted-foreground pt-8 border-t">
                    <p>© {new Date().getFullYear()} {appName}. All rights reserved.</p>
                </div>
            </div>
        </div>
    );
}
