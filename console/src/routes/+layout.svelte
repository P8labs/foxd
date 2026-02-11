<script lang="ts">
  import "../styles.css";
  import { page } from "$app/state";

  let { children } = $props();

  const navItems = [
    { path: "/", label: "Dashboard" },
    { path: "/devices", label: "Devices" },
    { path: "/rules", label: "Rules" },
    { path: "/logs", label: "Logs" },
    { path: "/config", label: "Configuration" },
  ];

  function isActive(path: string): boolean {
    return page.url.pathname === path;
  }
</script>

<svelte:head>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link
    rel="preconnect"
    href="https://fonts.gstatic.com"
    crossorigin="anonymous"
  />
  <link
    href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&display=swap"
    rel="stylesheet"
  />
</svelte:head>

<div class="app-container">
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="sidebar-title">FOXD</div>
      <div class="sidebar-subtitle">Network Monitor</div>
    </div>

    <nav>
      <ul class="sidebar-nav">
        {#each navItems as item}
          <li class="nav-item">
            <a
              href={item.path}
              class="nav-link"
              class:active={isActive(item.path)}
            >
              <span>{item.label}</span>
            </a>
          </li>
        {/each}
      </ul>
    </nav>
  </aside>

  <main class="main-content">
    {@render children()}
  </main>
</div>
