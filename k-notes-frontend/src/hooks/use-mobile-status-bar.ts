import { useEffect } from 'react';
import { StatusBar, Style } from '@capacitor/status-bar';
import { Capacitor } from '@capacitor/core';
import { useTheme } from 'next-themes';

export const useMobileStatusBar = () => {
    const { resolvedTheme } = useTheme();

    useEffect(() => {
        // Only run on native platforms
        if (Capacitor.isNativePlatform()) {
            const setStatusBarStyle = async () => {
                try {
                    // On Android, make sure the status bar overlays the WebView (transparent)
                    if (Capacitor.getPlatform() === 'android') {
                        await StatusBar.setOverlaysWebView({ overlay: true });
                    }

                    // Determine style based on theme
                    // If theme is dark => Use Dark style (usually white text)
                    // If theme is light => Use Light style (usually dark text)
                    const style = resolvedTheme === 'dark' ? Style.Dark : Style.Light;

                    await StatusBar.setStyle({ style });
                } catch (e) {
                    console.error('Failed to configure status bar:', e);
                }
            };

            setStatusBarStyle();
        }
    }, [resolvedTheme]);
};
