# Troubleshooting

This guide covers common issues and their solutions when running foxd.

## Daemon Won't Start

### Error: "Permission denied" when starting

**Symptom:**

```
Error: Failed to create packet capture: Permission denied
```

**Cause:** foxd needs elevated privileges to capture packets.

**Solutions:**

1. **Run with capabilities (recommended):**

   ```bash
   sudo setcap cap_net_raw,cap_net_admin=eip ./foxd
   ./foxd
   ```

2. **Run as root (not recommended):**

   ```bash
   sudo ./foxd
   ```

3. **Check capabilities were set:**
   ```bash
   getcap ./foxd
   # Should show: ./foxd = cap_net_admin,cap_net_raw+eip
   ```

### Error: "No such device" or "Interface not found"

**Symptom:**

```
Error: No such device: eth0
```

**Cause:** The configured network interface doesn't exist.

**Solution:**

1. **List available interfaces:**

   ```bash
   ip link show
   ```

2. **Update config.toml with correct interface:**

   ```toml
   [daemon]
   interface = "wlan0"  # or whichever interface exists
   ```

3. **Or use environment variable:**
   ```bash
   FOXD_INTERFACE=wlan0 ./foxd
   ```

### Error: "Address already in use"

**Symptom:**

```
Error: Failed to bind to 0.0.0.0:8080: Address already in use
```

**Cause:** Another process is using port 8080.

**Solutions:**

1. **Find what's using the port:**

   ```bash
   sudo lsof -i :8080
   sudo netstat -tulpn | grep 8080
   ```

2. **Stop the conflicting service or change foxd's port:**
   ```toml
   [api]
   port = 8081
   ```

### Database Error on Startup

**Symptom:**

```
Error: Failed to open database: database is locked
```

**Cause:** Another foxd instance is running or database file is corrupted.

**Solutions:**

1. **Check for running instances:**

   ```bash
   ps aux | grep foxd
   sudo systemctl status foxd
   ```

2. **Stop other instances:**

   ```bash
   sudo systemctl stop foxd
   killall foxd
   ```

3. **If database is corrupted, restore from backup or delete:**
   ```bash
   mv foxd.db foxd.db.backup
   # foxd will create a new database on next start
   ```

## Device Detection Issues

### No Devices Being Detected

**Symptom:** foxd starts successfully but no devices appear in the console.

**Diagnosis:**

1. **Check packet capture is working:**

   ```bash
   # Monitor logs for packet activity
   journalctl -u foxd -f | grep -i packet

   # Or if running in foreground, look for:
   # "Packet captured" or similar messages
   ```

2. **Verify interface has traffic:**

   ```bash
   sudo tcpdump -i eth0 -c 10
   ```

3. **Check metrics:**
   ```bash
   curl http://localhost:8080/api/metrics
   # Look at "packets_captured" - should be > 0
   ```

**Solutions:**

1. **Wrong interface:**
   - Verify you're monitoring the correct network interface
   - Use `ip link show` to list interfaces
   - Update `config.toml` with correct interface

2. **No ARP/DHCP traffic:**
   - Some networks (especially with switches) may not broadcast ARP to all ports
   - Try pinging devices to generate ARP traffic: `ping 192.168.1.1`
   - Check if you're on a VLAN or isolated network segment

3. **BPF filter too restrictive:**
   - Remove or adjust the `capture_filter` in config:

   ```toml
   [daemon]
   # Comment out or remove this line
   # capture_filter = "arp or (udp port 67 or udp port 68)"
   ```

4. **Interface not up:**
   ```bash
   sudo ip link set eth0 up
   ```

### Devices Detected But Immediately Go Offline

**Symptom:** Devices appear briefly then switch to offline status.

**Cause:** `device_timeout_secs` is set too low, or devices are actually inactive.

**Solutions:**

1. **Increase timeout:**

   ```toml
   [daemon]
   device_timeout_secs = 600  # 10 minutes instead of default
   ```

2. **Check neighbor table polling interval:**

   ```toml
   [daemon]
   neighbor_check_interval_secs = 30  # Check more frequently
   ```

3. **Verify devices are actually active:**
   ```bash
   ip neigh show
   # Look for your device's MAC address
   ```

### Devices Show Wrong IP or Hostname

**Symptom:** Device MAC address is correct but IP or hostname is wrong or missing.

**Cause:**

- Device changed IP address
- Hostname not broadcast via DHCP
- Stale ARP cache

**Solutions:**

1. **Wait for next update:** foxd will eventually see the correct information

2. **Force ARP refresh:**

   ```bash
   # From a device on the network
   ping 192.168.1.100  # IP of the device in question
   ```

3. **Check kernel neighbor table:**

   ```bash
   ip neigh show | grep aa:bb:cc:dd:ee:ff
   ```

4. **Use device nicknames:** If hostname is unreliable, set a nickname via the console or API

### Duplicate Devices

**Symptom:** Same device appears multiple times with different MACs.

**Cause:**

- Device has multiple network interfaces (Wi-Fi + Ethernet)
- MAC address randomization (privacy feature on phones)
- Virtual interfaces

**Solutions:**

1. **This is expected behavior** - foxd tracks by MAC address, which is per-interface
2. **Use nicknames** to identify which MAC corresponds to which device/interface
3. **MAC randomization:** Some modern phones randomize MAC addresses. Check device settings to disable this feature if needed.

## Notification Issues

### Notifications Not Being Sent

**Diagnosis Steps:**

1. **Check if rules are enabled:**

   ```bash
   curl http://localhost:8080/api/rules
   # Verify "enabled": true
   ```

2. **Check rule matches your event:**
   - Verify `trigger_type` matches the event
   - Check if `mac_filter` is restricting matches
   - Ensure device actually triggered the event

3. **Check notification channels exist:**

   ```bash
   curl http://localhost:8080/api/notifications
   # Verify channels are configured
   ```

4. **Check logs for errors:**

   ```bash
   journalctl -u foxd | grep -i notification
   curl http://localhost:8080/api/logs | jq '.logs[] | select(.category=="notification")'
   ```

5. **Verify metrics:**
   ```bash
   curl http://localhost:8080/api/metrics
   # Check "notifications_sent" increases
   ```

### Telegram Notifications Failing

**Symptoms:**

- No messages received
- Errors in logs about Telegram API

**Solutions:**

1. **Verify bot token:**

   ```bash
   TOKEN="your_bot_token"
   curl "https://api.telegram.org/bot${TOKEN}/getMe"
   # Should return bot information
   ```

2. **Verify chat ID:**

   ```bash
   # Send a message to your bot first, then:
   curl "https://api.telegram.org/bot${TOKEN}/getUpdates"
   # Look for "chat":{"id":123456789}
   ```

3. **Test sending manually:**

   ```bash
   TOKEN="your_bot_token"
   CHAT_ID="your_chat_id"
   curl -X POST "https://api.telegram.org/bot${TOKEN}/sendMessage" \
     -d "chat_id=${CHAT_ID}" \
     -d "text=Test from foxd"
   ```

4. **Check firewall/network:**

   ```bash
   curl -I https://api.telegram.org
   # Should return 200 or 302
   ```

5. **Update channel configuration:**
   - Delete and recreate the Telegram channel in foxd
   - Ensure no extra spaces in bot token or chat ID

### ntfy Notifications Failing

**Symptoms:**

- No notifications on ntfy app/website
- Connection errors in logs

**Solutions:**

1. **Test topic manually:**

   ```bash
   curl -d "Test message" https://ntfy.sh/your-topic
   ```

2. **Verify server URL:**
   - Must be full URL: `https://ntfy.sh`
   - For self-hosted: `https://ntfy.example.com`

3. **Check authentication:**
   - If topic requires auth, ensure token is set
   - Test with token:

   ```bash
   curl -H "Authorization: Bearer tk_xxxxxxxxx" \
     -d "Test" https://ntfy.sh/your-topic
   ```

4. **Network connectivity:**
   ```bash
   curl -I https://ntfy.sh
   ```

### Webhook Notifications Failing

**Symptoms:**

- Webhook endpoint not receiving data
- Connection errors in logs

**Solutions:**

1. **Test endpoint manually:**

   ```bash
   curl -X POST https://example.com/webhook \
     -H "Content-Type: application/json" \
     -d '{"test": "message"}'
   ```

2. **Check endpoint logs:** Verify your webhook server is running and accessible

3. **Verify URL is correct:** Must be full URL with protocol

4. **Test headers:**
   - If using authentication headers, verify they're correct
   - Headers should be JSON object:

   ```json
   {
     "Authorization": "Bearer token123"
   }
   ```

5. **Firewall/network:**
   - Ensure foxd can reach the webhook URL
   - Check for SSL certificate issues:
   ```bash
   curl -v https://your-webhook-url.com
   ```

## Performance Issues

### High CPU Usage

**Symptom:** foxd using >20% CPU continuously.

**Diagnosis:**

1. **Check packet capture rate:**
   ```bash
   curl http://localhost:8080/api/metrics | jq .packets_captured
   # Note the number, wait 10 seconds, check again
   ```

**Solutions:**

1. **Reduce packet capture scope with BPF filter:**

   ```toml
   [daemon]
   capture_filter = "arp or (udp port 67 or udp port 68)"
   ```

2. **Increase polling intervals:**

   ```toml
   [daemon]
   neighbor_check_interval_secs = 120  # Check less frequently
   ```

3. **Monitor on smaller network segment:** If possible, use a dedicated VLAN

4. **Check for packet flood:** Use `tcpdump` to see if there's unusual traffic

### High Memory Usage

**Symptom:** foxd using excessive RAM.

**Diagnosis:**

1. **Check number of devices:**

   ```bash
   curl http://localhost:8080/api/metrics | jq .total_devices
   ```

2. **Check logs count:**
   ```bash
   sqlite3 foxd.db "SELECT COUNT(*) FROM logs;"
   ```

**Solutions:**

1. **Enable log cleanup:**

   ```toml
   [daemon]
   log_cleanup_enabled = true
   log_retention_days = 7  # Keep less history
   ```

2. **Manually clean old logs:**

   ```bash
   sqlite3 foxd.db "DELETE FROM logs WHERE timestamp < datetime('now', '-7 days');"
   sqlite3 foxd.db "VACUUM;"
   ```

3. **Restart daemon:** Memory will be cleared

### Database Growing Large

**Symptom:** `foxd.db` file is many megabytes or gigabytes.

**Diagnosis:**

```bash
ls -lh foxd.db
sqlite3 foxd.db "SELECT name, COUNT(*) FROM sqlite_master JOIN (
  SELECT 'devices' as name UNION ALL
  SELECT 'logs' UNION ALL
  SELECT 'rules' UNION ALL
  SELECT 'notification_channels'
) USING(name) WHERE type='table' GROUP BY name;"
```

**Solutions:**

1. **Clean old logs:**

   ```bash
   sqlite3 foxd.db "DELETE FROM logs WHERE timestamp < datetime('now', '-30 days');"
   sqlite3 foxd.db "VACUUM;"
   ```

2. **Enable automatic cleanup:**

   ```toml
   [daemon]
   log_cleanup_enabled = true
   log_retention_days = 30
   ```

3. **Remove old offline devices:**

   ```bash
   # Backup first!
   cp foxd.db foxd.db.backup

   # Remove devices not seen in 90 days
   sqlite3 foxd.db "DELETE FROM devices WHERE status='offline'
     AND last_seen < datetime('now', '-90 days');"
   sqlite3 foxd.db "VACUUM;"
   ```

## Web Console Issues

### Console Not Loading

**Symptom:** Browser shows error or blank page.

**Diagnosis:**

1. **Check API is accessible:**

   ```bash
   curl http://localhost:8080/api/health
   ```

2. **Check browser console:** Open browser dev tools (F12) and look for errors

3. **Try different browser:** Rule out browser-specific issues

**Solutions:**

1. **Clear browser cache:** Hard refresh (Ctrl+Shift+R or Cmd+Shift+R)

2. **Rebuild console:**

   ```bash
   cd console
   pnpm install
   pnpm build
   cd ../daemon
   cargo build --release
   ```

3. **Check foxd was built with console:**
   ```bash
   # Console files should be embedded
   strings foxd | grep "index.html"
   ```

### Console Shows "Failed to fetch"

**Symptom:** Console loads but shows connection errors.

**Cause:** Browser cannot reach API (CORS, wrong URL, or daemon not running).

**Solutions:**

1. **Verify daemon is running:**

   ```bash
   curl http://localhost:8080/api/health
   ```

2. **Check browser URL matches daemon host/port:**
   - If daemon on `localhost:8080`, browser must access `http://localhost:8080`
   - If on remote host, use IP: `http://192.168.1.100:8080`

3. **Check firewall:**

   ```bash
   sudo ufw status
   sudo iptables -L
   ```

4. **Bind to correct interface:**
   ```toml
   [api]
   host = "0.0.0.0"  # Listen on all interfaces
   port = 8080
   ```

## Logging and Debugging

### Enable Verbose Logging

Set the `RUST_LOG` environment variable:

```bash
# Info level (default)
RUST_LOG=info ./foxd

# Debug level (verbose)
RUST_LOG=debug ./foxd

# Trace level (very verbose)
RUST_LOG=trace ./foxd

# Specific modules only
RUST_LOG=foxd_daemon=debug,axum=info ./foxd
```

For systemd service, edit the service file:

```ini
[Service]
Environment="RUST_LOG=debug"
ExecStart=/usr/local/bin/foxd
```

### View Logs in Real-Time

**If running as systemd service:**

```bash
# Follow logs
journalctl -u foxd -f

# Last 100 lines
journalctl -u foxd -n 100

# Filter by priority
journalctl -u foxd -p err  # Errors only
```

**If running in foreground:**
Logs print to stdout/stderr directly.

**Via API:**

```bash
curl http://localhost:8080/api/logs | jq .
```

### Common Log Messages

**"Packet captured: ARP request"**

- Normal operation, devices are being detected

**"Device timeout: aa:bb:cc:dd:ee:ff"**

- Device marked offline due to inactivity

**"Rule matched: [rule name]"**

- Notification rule triggered

**"Failed to send notification"**

- Notification delivery error, check channel configuration

**"Database query error"**

- SQLite error, may indicate corruption or locks

## Getting Help

If you're still experiencing issues:

1. **Check GitHub Issues:** [https://github.com/P8labs/foxd/issues](https://github.com/P8labs/foxd/issues)

2. **Create an issue with:**
   - foxd version (`./foxd --version`)
   - Operating system and version
   - Relevant configuration (redact tokens!)
   - Command used to start foxd
   - Error messages and logs
   - Steps to reproduce

3. **Provide debug logs:**

   ```bash
   RUST_LOG=debug ./foxd 2>&1 | tee foxd-debug.log
   # Run until issue occurs, then share foxd-debug.log (redact sensitive info)
   ```

4. **Check documentation:**
   - [Configuration Guide](./configuration.md)
   - [API Reference](../api/)
   - [Notification Setup](./notifications.md)
