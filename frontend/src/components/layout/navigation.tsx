"use client";

import { useState } from "react";
import Link from "next/link";
import { usePathname } from "next/navigation";
import { Button } from "@/components/ui/button";
import {
  Home,
  Image as ImageIcon,
  FolderOpen,
  Settings,
  LogOut,
  Menu,
  X,
  Brain,
  Search,
} from "lucide-react";

const navigationItems = [
  { href: "/", label: "ダッシュボード", icon: Home },
  { href: "/projects", label: "プロジェクト", icon: FolderOpen },
  { href: "/images", label: "画像管理", icon: ImageIcon },
  { href: "/ai-tools", label: "AI機能", icon: Brain },
  { href: "/search", label: "検索", icon: Search },
  { href: "/settings", label: "設定", icon: Settings },
];

export function Navigation() {
  const [isOpen, setIsOpen] = useState(false);
  const pathname = usePathname();

  return (
    <>
      {/* モバイル用ハンバーガーメニュー */}
      <div className="lg:hidden">
        <Button
          variant="ghost"
          size="sm"
          onClick={() => setIsOpen(!isOpen)}
          className="fixed top-4 left-4 z-50"
        >
          {isOpen ? <X className="h-6 w-6" /> : <Menu className="h-6 w-6" />}
        </Button>
      </div>

      {/* サイドバー */}
      <aside
        className={`
          fixed left-0 top-0 z-40 h-full w-64 bg-background border-r border-border
          transform transition-transform duration-300 ease-in-out
          lg:translate-x-0 lg:static lg:h-screen
          ${isOpen ? "translate-x-0" : "-translate-x-full"}
        `}
      >
        <div className="flex flex-col h-full">
          {/* ヘッダー */}
          <div className="p-6 border-b border-border">
            <h1 className="text-xl font-bold text-foreground">
              KG Annotation v2
            </h1>
            <p className="text-sm text-muted-foreground mt-1">
              Knowledge Graph アノテーション
            </p>
          </div>

          {/* ナビゲーション */}
          <nav className="flex-1 p-4">
            <ul className="space-y-2">
              {navigationItems.map((item) => {
                const Icon = item.icon;
                const isActive = pathname === item.href;
                
                return (
                  <li key={item.href}>
                    <Link
                      href={item.href}
                      onClick={() => setIsOpen(false)}
                      className={`
                        flex items-center px-3 py-2 rounded-md text-sm font-medium
                        transition-colors duration-200
                        ${
                          isActive
                            ? "bg-primary text-primary-foreground"
                            : "text-muted-foreground hover:text-foreground hover:bg-muted"
                        }
                      `}
                    >
                      <Icon className="mr-3 h-5 w-5" />
                      {item.label}
                    </Link>
                  </li>
                );
              })}
            </ul>
          </nav>

          {/* ユーザーメニュー */}
          <div className="p-4 border-t border-border">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">ユーザー名</p>
                <p className="text-xs text-muted-foreground">user@example.com</p>
              </div>
              <Button variant="ghost" size="sm">
                <LogOut className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      </aside>

      {/* モバイル用オーバーレイ */}
      {isOpen && (
        <div
          className="fixed inset-0 z-30 bg-black bg-opacity-50 lg:hidden"
          onClick={() => setIsOpen(false)}
        />
      )}
    </>
  );
}
