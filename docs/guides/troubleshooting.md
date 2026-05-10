# Troubleshooting

> Fix the most common issues in under 30 seconds.

## Quick Fix: The Universal Reset

If something is broken and you don't want to debug:

```bash
tytus disconnect
tytus connect
tytus test
```

This tears down the tunnel, reconnects, and verifies everything works. Fixes 90% of issues.

---

## Common Problems

### macOS says “unidentified developer” or blocks the pkg

**What happened:** The public beta pkg is intentionally unsigned while Apple Developer ID signing/notarization is deferred.

**Fix:**
1. Make sure the filename contains `PUBLIC-BETA-UNSIGNED`.
2. Control-click the pkg in Finder.
3. Choose **Open**.
4. Confirm the warning.
5. After install, open **Tytus** and follow the setup wizard.

Do not use this as a GA security model. Signed/notarized pkg is still the production gate.

---

### Linux says the deb is unsigned

**What happened:** The public beta deb is intentionally unsigned while package/repository signing is deferred.

**Fix:** Only install the deb from the official beta page or GitHub release, verify it matches `v0.6.14-beta.33`, then install:

```bash
sudo apt install ./Tytus-0.6.14-x86_64-unknown-linux-gnu-unsigned-PUBLIC-BETA-UNSIGNED.deb
```

Open Tytus from the launcher or run `tytus-tray`, then follow the setup wizard.

---

### I installed the beta but do not see Tytus

**Fix:**

```bash
which tytus
which tytus-tray
tytus --version
tytus-tray
```

On macOS, also try:

```bash
open -a Tytus
```

If `tytus-tray` starts, use the setup wizard. If it does not, run `tytus doctor` and include the output in the issue.

---

### "Not logged in. Run: tytus login"

**What happened:** Your session expired, or you've never logged in on this machine.

**Fix:**
```bash
tytus login
```
A browser window opens. Sign in, and you're back.

---

### "No Tytus subscription. Upgrade at traylinx.com"

**What happened:** Your account doesn't have an active Tytus plan, or the credentials are stale.

**Fix:**
1. Check your subscription at [traylinx.com](https://traylinx.com)
2. If you do have a plan, try logging in again:
   ```bash
   tytus logout
   tytus login
   ```

---

### "Token refresh failed: AuthExpired"

**What happened:** Your login session fully expired and the automatic refresh didn't work.

**Fix:**
```bash
tytus login
```
This gets a fresh session. Then reconnect:
```bash
tytus connect
```

---

### Tunnel Up But curl Times Out

**What happened:** The tunnel process is running but traffic isn't flowing. Usually caused by:
- Another VPN interfering with routing
- WiFi switched and the tunnel didn't recover
- The tunnel daemon died but `tytus status` shows it as active

**Fix:**
```bash
# Step 1: Check the real state
tytus doctor

# Step 2: Reconnect
tytus disconnect
tytus connect

# Step 3: Test
tytus test
```

If you're running another VPN (Tailscale, WireGuard, corporate VPN), try disconnecting it first. The VPN may be capturing the traffic meant for your pod.

---

### "403 plan_limit_reached"

**What happened:** You tried to allocate a pod but your plan doesn't have enough units left.

**Fix:** Either free an existing pod or upgrade:
```bash
# See what's allocated
tytus status

# Free a pod (DESTRUCTIVE — the pod is deleted)
tytus revoke <pod_id>

# Now connect again
tytus connect
```

---

### "Tunnel daemon already running"

**What happened:** A previous `tytus connect` left a tunnel process running.

**Fix:**
```bash
tytus disconnect
tytus connect
```

---

### "Pod config not ready" (after 30 seconds)

**What happened:** Your pod's server is still booting up. This happens when a fresh server is being provisioned.

**Fix:** Wait 60 seconds and try again:
```bash
tytus connect
```

If it keeps happening, the server may have an issue. Contact support.

---

### Autostart Not Working After Reboot

**What happened:** The LaunchAgent is installed but the tunnel doesn't come up after reboot.

**Fix:**
```bash
# Check if autostart is installed
tytus autostart status

# Check the diagnostic log
cat /tmp/tytus/autostart.log

# Common cause: login session expired
# Fix: re-login, then the next reboot will work
tytus login
```

---

### "401 Invalid API key" from the Gateway

**What happened:** Your stable API key hasn't synced to the pod yet. This usually happens right after first connect.

**Fix:** Wait 2-3 seconds and retry. If it persists:
```bash
tytus restart
```


### TytusOS shows "Session expired" but the tray says connected

**What happened:** Your browser/daemon state is stale or your Sentinel refresh session expired. Pods can keep running while the local UI needs re-authentication.

**Fix:**
```bash
tytus login
tytus status
```
Then click **Check session** in Settings → Daemon or reload TytusOS. Do not revoke pods for this.

---

### Pod Inspector says "Not ready" but the agent URL opens

**What happened:** The pod has a public URL, but one readiness layer failed, usually bootstrap smoke health or optional shared storage helper.

**Fix:** Open the pod tab in Pod Inspector and read the readiness rows. If core HTTP/UI is green and only shared storage is degraded, the agent can still be used. If bootstrap smoke is red, run:
```bash
tytus doctor --pod NN
tytus restart --pod NN
```

---

### Files shows raw `tytus ls: no such path` for inbox

**What happened:** The pod has not created `/app/workspace/inbox` yet. This should render as an empty state in TytusOS.

**Fix:** Create the folder or push the first file:
```bash
tytus exec --pod NN "mkdir -p /app/workspace/inbox"
tytus push ./example.txt --pod NN
```

---
## The Full Diagnostic

When nothing else works:

```bash
tytus doctor
```

This checks:
1. Are you logged in?
2. Is your token valid?
3. Do you have a subscription?
4. Are pods allocated?
5. Is the tunnel running?
6. Is the gateway reachable?
7. Can you send a chat completion?
8. Is the MCP server configured?

Each check reports pass/fail with specific guidance.

---

## Getting Debug Logs

For deep debugging, enable verbose logging:

```bash
RUST_LOG=debug tytus connect
```

Or check the tunnel daemon's log:
```bash
cat /tmp/tytus/tunnel-02.log
```

Or the autostart diagnostic log:
```bash
cat /tmp/tytus/autostart.log
```

---

## Contact

If `tytus doctor` can't solve it, reach out:
- **GitHub Issues**: [traylinx/tytus-cli](https://github.com/traylinx/tytus-cli/issues)
- **Email**: hello@traylinx.com
