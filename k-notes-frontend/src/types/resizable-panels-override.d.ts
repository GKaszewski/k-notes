declare module "react-resizable-panels" {
    import * as React from "react";

    export interface PanelGroupProps extends React.HTMLAttributes<HTMLDivElement> {
        direction: "horizontal" | "vertical";
        autoSaveId?: string;
        storage?: any;
    }
    export const PanelGroup: React.FC<PanelGroupProps>;

    export interface PanelProps extends React.HTMLAttributes<HTMLDivElement> {
        defaultSize?: number;
        minSize?: number;
        maxSize?: number;
        order?: number;
        collapsible?: boolean;
        collapsedSize?: number;
        onCollapse?: (collapsed: boolean) => void;
        onResize?: (size: number) => void;
    }
    export const Panel: React.FC<PanelProps>;

    export interface PanelResizeHandleProps extends React.HTMLAttributes<HTMLDivElement> {
        disabled?: boolean;
        hitAreaMargins?: { fine: number; coarse: number };
    }
    export const PanelResizeHandle: React.FC<PanelResizeHandleProps>;
}
