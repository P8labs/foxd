# Fox Daemon Console

A modern web dashboard for monitoring and managing the Fox Daemon LAN monitoring system.

## Features

- **Real-time Dashboard** - View live device counts, status,  and system metrics
- **Device Management** - Monitor all network devices with filtering and search
- **Rule Configuration** - Create and manage notification rules with an intuitive UI
- **Configuration Panel** - Manage daemon settings and notification channels
- **Component-Based Architecture** - Modular, maintainable Svelte components
- **Responsive Design** - Works on desktop, tablet, and mobile devices
- **Auto-Refresh** - Real-time updates without page reload

## Tech Stack

- **Framework**: SvelteKit 2.0+
- **Language**: TypeScript
- **Styling**: Custom CSS (No Tailwind)
- **Build Tool**: Vite
- **HTTP Client**: Fetch API

## Quick Start

### Prerequisites

- Node.js 18+ and npm/pnpm
- Fox Daemon running and accessible on port 8080

### Installation

```bash
cd console
npm install
```

### Configuration

1. Copy the environment file:
```bash
cp .env.example .env
```

2. Edit `.env` and set your API URL:
```env
VITE_API_URL=http://localhost:8080
```

### Development

Start the development server:
```bash
npm run dev
```

The console will be available at `http://localhost:5173`

### Production Build

Build the application:
```bash
npm run build
```

Preview the production build:
```bash
npm run preview
```

The built files will be in the `build/` directory.

## Project Structure

```
console/
├── src/
│   ├── lib/
│   │   ├── api.ts              # API client and types
│   │   └── components/         # Reusable UI components
│   │       ├── Alert.svelte    # Alert notifications
│   │       ├── Loading.svelte  # Loading spinner
│   │       ├── Modal.svelte    # Modal dialog
│   │       └── StatCard.svelte # Statistics card
│   ├── routes/
│   │   ├── +layout.svelte      # App layout with sidebar
│   │   ├── +page.svelte        # Dashboard page
│   │   ├── devices/            # Devices page
│   │   ├── rules/              # Rules page
│   │   └── config/             # Configuration page
│   ├── styles.css              # Global styles
│   └── app.html                # HTML template
├── package.json
├── tsconfig.json
├── vite.config.ts
└── README.md
```

## Component Overview

### Pages

#### Dashboard (`/`)
- System health indicator
- Key metrics (devices, rules, uptime)
- Network activity statistics
- Recent devices list
- Auto-refreshes every 30 seconds

#### Devices (`/devices`)
- Complete device list with search
- Filter by status (online/offline)
- Real-time status updates
- Auto-refreshes every 15 seconds

#### Rules (`/rules`)
- List all notification rules
- Create/edit/delete rules
- Configure triggers and channels
- Enable/disable rules

#### Configuration (`/config`)
- View daemon settings
- Manage notification channels
- Add Telegram, ntfy, or webhook channels
- Remove existing channels

### Components

#### `StatCard.svelte`
Displays a single statistic with icon, value, and subtitle.

```svelte
<StatCard
  title="Online Devices"
  icon="🟢"
  value={15}
  subtitle="75% of total"
/>
```

#### `Alert.svelte`
Shows success, error, warning, or info messages.

```svelte
<Alert type="success" message="Device added successfully" />
```

#### `Loading.svelte`
Displays a loading spinner with optional message.

```svelte
<Loading message="Loading devices..." />
```

#### `Modal.svelte`
Generic modal dialog with header, body, and close functionality.

```svelte
<Modal bind:open={showModal} title="Add Rule" onClose={() => showModal = false}>
  <!-- Modal content -->
</Modal>
```

## API Integration

The console communicates with the Fox Daemon API via the `api.ts` service:

```typescript
import { api } from '$lib/api';

// Get devices
const devices = await api.getDevices();

// Create a rule
await api.createRule({
  name: "New Device Alert",
  trigger_type: "new_device",
  enabled: true,
  notification_channels: ["telegram_123"]
});

// Update config
await api.updateConfig({
  notifications: [/* channels */]
});
```

## Styling

The console uses custom CSS without any CSS framework. All styles are defined in `src/styles.css` with:

- CSS Custom Properties for theming
- BEM-like naming conventions
- Responsive design with media queries
- Reusable utility classes

### Color Scheme

The default theme uses:
- Primary: `#1a1a1a` (dark gray)
- Success: `#10b981` (green)
- Danger: `#ef4444` (red)
- Warning: `#f59e0b` (orange)
- Info: `#3b82f6` (blue)

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `VITE_API_URL` | Fox Daemon API base URL | `http://localhost:8080` |

## Deployment

### Node.js Server

The console uses `@sveltejs/adapter-node` for deployment:

```bash
npm run build
node build
```

Set the `PORT` environment variable:
```bash
PORT=3000 node build
```

### Static Hosting

To deploy as static files, change the adapter in `svelte.config.js`:

```javascript
import adapter from '@sveltejs/adapter-static';
```

Then build:
```bash
npm run build
```

Upload the `build/` directory to your static host.

### Docker

Create a `Dockerfile`:

```dockerfile
FROM node:20-alpine as build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine
WORKDIR /app
COPY --from=build /app/build build/
COPY --from=build /app/package.json .
RUN npm ci --production
EXPOSE 3000
CMD ["node", "build"]
```

Build and run:
```bash
docker build -t foxd-console .
docker run -p 3000:3000 -e VITE_API_URL=http://your-daemon:8080 foxd-console
```

### Reverse Proxy

Example nginx configuration:

```nginx
server {
    listen 80;
    server_name console.example.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }
}
```

## Development

### Type Checking

Run TypeScript type checking:
```bash
npm run check
```

Watch mode:
```bash
npm run check:watch
```

### Code Organization

- Keep components small and focused
- Use TypeScript for type safety
- Follow Svelte 5 runes syntax ($state, $derived, etc.)
- Place reusable logic in `$lib/`
- Use semantic HTML

### Adding a New Page

1. Create directory in `src/routes/`:
```bash
mkdir src/routes/newpage
```

2. Create `+page.svelte`:
```svelte
<script lang="ts">
  // Your page logic
</script>

<h1>New Page</h1>
```

3. Add to navigation in `+layout.svelte`:
```typescript
const navItems = [
  // ...
  { path: '/newpage', label: 'New Page', icon: '📄' }
];
```

## Troubleshooting

### Console can't connect to API

- Check that Fox Daemon is running on port 8080
- Verify `VITE_API_URL` in `.env`
- Check browser console for CORS errors
- Ensure daemon API is accessible from your machine

### Blank page after build

- Check build output for errors
- Verify `adapter-node` is installed
- Check Node.js version (requires 18+)

### Styles not loading

- Ensure `src/styles.css` is imported in `+layout.svelte`
- Clear browser cache
- Check for CSS syntax errors

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## License

MIT License - See LICENSE file

## Contributing

Contributions welcome! Please follow the existing code style and component patterns.

## Support

For issues or questions, please visit the Fox Daemon repository.
