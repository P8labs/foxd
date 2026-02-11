import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Foxd",
  description:
    "A local-first LAN monitoring daemon that tracks device presence, exposes a REST API, and sends real-time notifications.",
  head: [["link", { rel: "icon", href: "/favicon.png" }]],
  themeConfig: {
    logo: "/favicon.png",
    nav: [
      { text: "Home", link: "/" },
      { text: "Guide", link: "/guide/getting-started" },
      { text: "API", link: "/api/" },
      { text: "GitHub", link: "https://github.com/P8labs/foxd" },
    ],

    sidebar: [
      {
        text: "Introduction",
        items: [
          { text: "What is foxd?", link: "/" },
          { text: "Getting Started", link: "/guide/getting-started" },
        ],
      },
      {
        text: "Guides",
        items: [
          { text: "Configuration", link: "/guide/configuration" },
          { text: "Device Tracking", link: "/guide/devices" },
          { text: "Rules & Automation", link: "/guide/rules" },
          { text: "Notifications", link: "/guide/notifications" },
        ],
      },
      {
        text: "API Reference",
        items: [
          { text: "Overview", link: "/api/" },
          { text: "Devices API", link: "/api/devices" },
          { text: "Rules API", link: "/api/rules" },
          { text: "Config API", link: "/api/config" },
        ],
      },
    ],

    socialLinks: [{ icon: "github", link: "https://github.com/P8labs/foxd" }],

    footer: {
      copyright: "© 2026 P8labs",
    },
  },
});
