import { Home, Archive, Settings, Tag, ChevronRight, Pencil, Trash2, MoreHorizontal } from "lucide-react"
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { Link, useLocation, useSearchParams, useNavigate } from "react-router-dom"
import { SettingsDialog } from "@/components/settings-dialog"
import { useState } from "react"
import { useTags, useDeleteTag, useRenameTag } from "@/hooks/use-notes"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Badge } from "@/components/ui/badge"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { toast } from "sonner"
import { useTranslation } from "react-i18next"

const items = [
  {
    titleKey: "Notes",
    url: "/",
    icon: Home,
  },
  {
    titleKey: "Archive",
    url: "/archive",
    icon: Archive,
  },
]

interface TagItemProps {
  tag: { id: string; name: string };
  isActive: boolean;
}

function TagItem({ tag, isActive }: TagItemProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editName, setEditName] = useState(tag.name);
  const { mutate: deleteTag } = useDeleteTag();
  const { mutate: renameTag } = useRenameTag();
  const navigate = useNavigate();
  const { t } = useTranslation();

  const handleDelete = () => {
    if (confirm(t("Delete tag \"{{name}}\"? Notes will keep their content.", { name: tag.name }))) {
      deleteTag(tag.id, {
        onSuccess: () => {
          toast.success(t("Tag deleted"));
          navigate("/");
        },
        onError: (err: any) => toast.error(err.message)
      });
    }
  };

  const handleRename = () => {
    if (editName.trim() && editName.trim() !== tag.name) {
      renameTag({ id: tag.id, name: editName.trim() }, {
        onSuccess: () => {
          toast.success(t("Tag renamed"));
          setIsEditing(false);
        },
        onError: (err: any) => {
          toast.error(err.message);
          setEditName(tag.name);
        }
      });
    } else {
      setIsEditing(false);
      setEditName(tag.name);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleRename();
    } else if (e.key === "Escape") {
      setIsEditing(false);
      setEditName(tag.name);
    }
  };

  if (isEditing) {
    return (
      <SidebarMenuItem>
        <Input
          value={editName}
          onChange={(e) => setEditName(e.target.value)}
          onBlur={handleRename}
          onKeyDown={handleKeyDown}
          autoFocus
          className="h-7 text-xs"
        />
      </SidebarMenuItem>
    );
  }

  return (
    <SidebarMenuItem className="group/tag">
      <SidebarMenuButton
        asChild
        isActive={isActive}
        tooltip={tag.name}
        className="pr-0"
      >
        <Link to={`/?tag=${encodeURIComponent(tag.name)}`} className="flex items-center justify-between w-full">
          <Badge variant="secondary" className="text-xs px-1.5 py-0">
            {tag.name}
          </Badge>
          <DropdownMenu>
            <DropdownMenuTrigger asChild onClick={(e) => e.preventDefault()}>
              <Button
                variant="ghost"
                size="icon"
                className="h-6 w-6 opacity-0 group-hover/tag:opacity-100 transition-opacity"
              >
                <MoreHorizontal className="h-3.5 w-3.5" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-32">
              <DropdownMenuItem onClick={(e) => { e.preventDefault(); setIsEditing(true); }}>
                <Pencil className="mr-2 h-3.5 w-3.5" />
                {t("Rename")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={(e) => { e.preventDefault(); handleDelete(); }} className="text-destructive focus:text-destructive">
                <Trash2 className="mr-2 h-3.5 w-3.5" />
                {t("Delete")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </Link>
      </SidebarMenuButton>
    </SidebarMenuItem>
  );
}

export function AppSidebar() {
  const location = useLocation();
  const [searchParams] = useSearchParams();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [tagsOpen, setTagsOpen] = useState(true);
  const { t } = useTranslation();

  const { data: tags } = useTags();
  const activeTag = searchParams.get("tag");

  return (
    <>
      <Sidebar collapsible="icon">
        <SidebarContent>
          <SidebarGroup>
            <SidebarGroupLabel>{t("K-Notes")}</SidebarGroupLabel>
            <SidebarGroupContent>
              <SidebarMenu>
                {items.map((item) => (
                  <SidebarMenuItem key={item.titleKey}>
                    <SidebarMenuButton asChild isActive={location.pathname === item.url && !activeTag} tooltip={t(item.titleKey)}>
                      <Link to={item.url}>
                        <item.icon />
                        <span>{t(item.titleKey)}</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                ))}

                <SidebarMenuItem>
                  <SidebarMenuButton onClick={() => setSettingsOpen(true)} tooltip={t("Settings")}>
                    <Settings />
                    <span>{t("Settings")}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroupContent>
          </SidebarGroup>

          {/* Tag Browser Section */}
          <SidebarGroup>
            <Collapsible open={tagsOpen} onOpenChange={setTagsOpen}>
              <SidebarGroupLabel asChild>
                <CollapsibleTrigger className="flex items-center justify-between w-full cursor-pointer group/collapsible">
                  <div className="flex items-center gap-1.5">
                    <Tag className="h-3.5 w-3.5" />
                    <span>{t("Tags")}</span>
                  </div>
                  <ChevronRight className="h-3.5 w-3.5 transition-transform group-data-[state=open]/collapsible:rotate-90" />
                </CollapsibleTrigger>
              </SidebarGroupLabel>
              <CollapsibleContent>
                <SidebarGroupContent>
                  <ScrollArea className="max-h-48">
                    <SidebarMenu>
                      {tags && tags.length > 0 ? (
                        tags.map((tag: { id: string; name: string }) => (
                          <TagItem
                            key={tag.id}
                            tag={tag}
                            isActive={activeTag === tag.name}
                          />
                        ))
                      ) : (
                        <div className="px-2 py-1.5 text-xs text-muted-foreground">
                          {t("No tags yet")}
                        </div>
                      )}
                    </SidebarMenu>
                  </ScrollArea>
                </SidebarGroupContent>
              </CollapsibleContent>
            </Collapsible>
          </SidebarGroup>
        </SidebarContent>
      </Sidebar>
      <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} dataManagementEnabled />
    </>
  )
}

