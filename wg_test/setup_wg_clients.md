Below is a **complete, well-structured guide** for **client-side WireGuard setup**, covering **two major cases**:

---

# ✅ **PART 1 — Kernel-level WireGuard Clients**

## Platforms covered:

* **Ubuntu / Debian / Linux** → *Kernel WireGuard available*
* **macOS** → *Kernel WireGuard **NOT** available — always uses userspace Engine (`wireguard-go`) inside the official app*

So the differences are explained clearly.

---

# ⭐ PART 1A — **Client Setup (Kernel WireGuard) on Ubuntu / Debian / Linux**

Linux distributions with Kernel 5.6+ include WireGuard **built into the kernel**.
This is the **fastest and most stable** implementation.

---

## ✔ **Step 1 — Install WireGuard**

```bash
sudo apt update
sudo apt install wireguard
```

This installs:

* kernel wireguard module
* `wg` and `wg-quick` tools
* systemd integration

---

## ✔ **Step 2 — Create client keys**

```bash
wg genkey | tee privatekey | wg pubkey > publickey
```

You'll use:

* `privatekey` → in `[Interface]`
* `publickey` → given to VPS

---

## ✔ **Step 3 — Create client config (`/etc/wireguard/wg0.conf`)**

Example for **Client A**:

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

Same model works for any number of clients.

---

## ✔ **Step 4 — Start WireGuard**

```bash
sudo wg-quick up wg0
```

Enable on boot:

```bash
sudo systemctl enable wg-quick@wg0
```

---

## ✔ **Step 5 — Testing**

```bash
ping 10.0.0.1   # VPS
ping 10.0.0.3   # Another peer
```

---

# ⭐ Important: **Kernel WireGuard on macOS does NOT exist**

macOS WireGuard works, but **not using kernel WireGuard**.
Apple does not allow third-party kernel modules.

macOS uses:

* **userspace WireGuard** implemented via `wireguard-go`
* The official WireGuard.app bundles everything

So for macOS clients, use Part 2 below.

---

# ⭐ PART 1B — **Client Setup on macOS (Userspace, but Official)**

Even though it’s *userspace*, the **official WireGuard macOS app** is fast and stable.

---

## ✔ Step 1 — Install WireGuard app

Download from:

[https://www.wireguard.com/install/](https://www.wireguard.com/install/)

Or via Homebrew:

```bash
brew install --cask wireguard
```

---

## ✔ Step 2 — Add your config

Create a `.conf` file like:

```ini
[Interface]
Address = 10.0.0.10/32
PrivateKey = <MAC_PRIVATE_KEY>

[Peer]
PublicKey = <SERVER_PUBLIC_KEY>
Endpoint = <VPS_PUBLIC_IP>:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

Import it in the app.

---

## ✔ Step 3 — Activate tunnel

* Open menu bar → WireGuard → Activate

macOS now behaves like any WireGuard peer.

---

# ⭐ Summary of Kernel vs macOS

| OS                      | Kernel WG? | Userspace?           | Notes                                  |
| ----------------------- | ---------- | -------------------- | -------------------------------------- |
| Ubuntu / Debian / Linux | ✔ Yes      | Optional             | Fastest, highest throughput            |
| macOS                   | ❌ No       | ✔ Yes (wireguard-go) | Official WG client uses userspace only |
| Windows                 | ❌ No       | ✔ Yes                | Uses wireguard-go backend              |

---

# ⭐ PART 2 — **Userspace WireGuard Clients (wireguard-rs or wireguard-go)**

Use this when:

* Your device OS does **not support kernel WG**
* You want a portable or embedded implementation
* You want WireGuard on unusual environments (containers, NAS, BSD, etc.)

---

# ⭐ PART 2A — Using `wireguard-rs`

`wireguard-rs` is a **Rust userspace implementation**, high quality and compatible.

It uses a **TUN device**, so root access is required.

---

## ✔ Step 1 — Install wireguard-rs

Depending on distribution, you may:

* Build from source
* Download binary from DefGuard releases
* Use package manager if available

Example (generic):

```bash
git clone https://github.com/DefGuard/wireguard-rs
cd wireguard-rs
cargo build --release
```

Executable will be under:

```
target/release/wireguard-rs
```

---

## ✔ Step 2 — Create WireGuard config

Same format as normal WireGuard.

`/etc/wireguard/wg0.conf`:

```ini
[Interface]
Address = 10.0.0.2/32
PrivateKey = <CLIENT_PRIVATE_KEY>

[Peer]
PublicKey = <SERVER_PUBLIC_KEY>
Endpoint = <VPS_PUBLIC_IP>:51820
AllowedIPs = 10.0.0.0/24
PersistentKeepalive = 25
```

---

## ✔ Step 3 — Start wireguard-rs with the config

Exact commands vary by version.
A common invocation:

```bash
sudo ./wireguard-rs -f /etc/wireguard/wg0.conf
```

Some builds may use:

```bash
sudo wg-quick up wg0
```

because wireguard-rs can act as a backend.

---

## ✔ Step 4 — Confirm tunnel is up

```bash
ip a
ping 10.0.0.1
wg show
```

---

# ⭐ PART 2B — Using `wireguard-go` (common fallback)

This is WireGuard’s official userspace “engine.”

Install:

```bash
sudo apt install wireguard-go
```

Then:

```bash
sudo wg-quick up wg0
```

This uses:

* `wg0.conf`
* Userspace implementation behind the scenes

---

# ⭐ When to choose userspace vs kernel

| Use Case              | Recommendation                                      |
| --------------------- | --------------------------------------------------- |
| VPS with Linux        | **Kernel** (fastest)                                |
| Server with BSD       | Use **wireguard-go** or **wireguard-rs**            |
| macOS                 | Must use **wireguard-go** via the app               |
| Windows               | Official client uses **wireguard-go**               |
| Small embedded device | **wireguard-rs** is ideal                           |
| Docker containers     | Userspace WG can run without privileged kernel mods |

---

# ⭐ Final Summary

## ✔ Ubuntu/Linux clients

* Fully kernel-level WireGuard
* Fastest performance
* Uses `wg` and `wg-quick`

## ✔ macOS clients

* Always userspace (`wireguard-go`)
* Use WireGuard.app

## ✔ Userspace implementations

* **wireguard-rs** (Rust, modern)
* **wireguard-go** (official fallback)
* Use TUN interfaces
* Suitable where kernel WG is unavailable

Both interoperate 100% with a kernel WireGuard VPS.

---

# ⭐ If you want, I can create:

* A combined PDF for server + kernel clients + userspace clients
* A fully filled config set for your environment
* Systemd service units for auto-starting wireguard-rs
* Diagrams for multi-device CGNAT design

Just tell me what format you want!

