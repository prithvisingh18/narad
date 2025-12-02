# AI generated

Below is a **clean, complete, step-by-step guide** on how to set up **WireGuard on a VPS** (Linux server).
This is the *hub* for connecting devices behind CGNAT.

The instructions are written for **Ubuntu/Debian**, but they work almost identically for most Linux distros.

---

# ⭐ **1. Install WireGuard on the VPS**

Update packages:

```bash
sudo apt update
sudo apt install wireguard
```

Confirm that the WireGuard kernel module exists:

```bash
modprobe wireguard
```

If no errors appear → it’s ready.

---

# ⭐ **2. Create WireGuard configuration directory**

```bash
sudo mkdir -p /etc/wireguard
sudo chmod 700 /etc/wireguard
```

---

# ⭐ **3. Generate server keys**

```bash
wg genkey | tee server_private.key | wg pubkey > server_public.key
```

Check files:

```bash
cat server_private.key
cat server_public.key
```

Keep `server_private.key` **secret**.

---

# ⭐ **4. Choose your VPN subnet**

A common WireGuard VPN range is:

```
10.0.0.0/24
```

We'll assign:

* VPS = **10.0.0.1**
* Client A = **10.0.0.2**
* Client B = **10.0.0.3**
* More clients get more /32s

---

# ⭐ **5. Create the server config (`/etc/wireguard/wg0.conf`)**

```ini
[Interface]
Address = 10.0.0.1/24
ListenPort = 51820
PrivateKey = <SERVER_PRIVATE_KEY>

# Peer A
[Peer]
PublicKey = <A_PUBLIC_KEY>
AllowedIPs = 10.0.0.2/32

# Peer B
[Peer]
PublicKey = <B_PUBLIC_KEY>
AllowedIPs = 10.0.0.3/32
```

Replace placeholders with real keys.

---

# ⭐ **6. Enable IP forwarding (routing)**

WireGuard requires the VPS to act as a router.

Temporary:

```bash
sudo sysctl -w net.ipv4.ip_forward=1
```

Permanent:

Edit `/etc/sysctl.conf` and add:

```
net.ipv4.ip_forward=1
```

Apply changes:

```bash
sudo sysctl -p
```

---

# ⭐ **7. (Optional) Allow Clients to Use VPS as an Internet Exit Node**

ONLY if you want VPN clients to browse the internet through the VPS.

Enable NAT:

```bash
sudo iptables -t nat -a postrouting -o eth0 -j masquerade
```

Persist rules:

```bash
sudo apt install iptables-persistent
sudo netfilter-persistent save
```

Replace **eth0** with your VPS’s internet-facing NIC (check with `ip a`).

---

# ⭐ **8. Start WireGuard**

```bash
sudo wg-quick up wg0
```

To make it start on boot:

```bash
sudo systemctl enable wg-quick@wg0
```

---

# ⭐ **9. Open firewall (if using UFW)**

Allow WireGuard’s UDP port:

```bash
sudo ufw allow 51820/udp
```

---

# ⭐ **10. Create client configs**

### **Client A config**

Safe to put in `/etc/wireguard/wg0.conf` on your device:

```ini
[Interface]
Address = 10.0.0.2/32
PrivateKey = <A_PRIVATE_KEY>
DNS = 1.1.1.1

[Peer]
PublicKey = <SERVER_PUBLIC_KEY>
Endpoint = <VPS_PUBLIC_IP>:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

### **Client B config**

```ini
[Interface]
Address = 10.0.0.3/32
PrivateKey = <B_PRIVATE_KEY>
DNS = 1.1.1.1

[Peer]
PublicKey = <SERVER_PUBLIC_KEY>
Endpoint = <VPS_PUBLIC_IP>:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

---

# ⭐ **11. Start WireGuard on the clients**

Linux:

```bash
sudo wg-quick up wg0
```

Windows/macOS: import the config into the WireGuard GUI.

---

# ⭐ **12. Test connectivity**

From Client A:

```bash
ping 10.0.0.1   # Ping the VPS
ping 10.0.0.3   # Ping Client B
```

From Client B:

```bash
ping 10.0.0.2
```

Check active peers on the VPS:

```bash
sudo wg
```

You should see received bytes from both peers.

---

# ⭐ **13. Now your VPS is a CGNAT-proof hub**

✔ Both clients behind CGNAT can talk to the VPS
✔ Both clients can talk to each other (10.0.0.2 ↔ 10.0.0.3)
✔ You can host FTP/SSH/HTTP on either device
✔ Bind services to WG IPs (e.g., 10.0.0.2) for private access only
✔ VPS encrypts & routes everything at kernel speeds

---

# ⭐ **If you want, I can prepare next:**

### ✔ A complete "multi-device behind CGNAT" architecture

### ✔ WireGuard-rs versions of the configs

### ✔ A version for IPv6

### ✔ A version for multiple LAN networks

### ✔ A PDF or DOCX of these instructions

### ✔ A graphical diagram of the entire system

Just tell me!

