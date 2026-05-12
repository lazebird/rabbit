//! Network interface utilities

use schema::NetworkProvider;
use std::collections::HashMap;
use std::net::Ipv4Addr;

/// Default implementation of [`schema::NetworkProvider`].
pub struct DefaultNetworkProvider;

impl NetworkProvider for DefaultNetworkProvider {
    fn get_mac_from_arp(&self, ip: Ipv4Addr) -> Option<String> {
        get_mac_from_arp(ip)
    }

    fn get_local_mac_table(&self) -> HashMap<Ipv4Addr, String> {
        get_local_ip_mac_table()
    }
}

/// Get MAC address from kernel ARP table.
///
/// - Linux: `SIOCGARP` ioctl with correct interface name (from `getifaddrs`)
/// - Windows: `SendARP`
///
/// Returns `None` if the MAC could not be resolved.
#[cfg_attr(target_os = "linux", allow(unreachable_code))]
pub fn get_mac_from_arp(ip: Ipv4Addr) -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        return get_mac_via_ioctl(ip);
    }
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::NetworkManagement::IpHelper::SendARP;

        let dest_ip: u32 = u32::from_be_bytes(ip.octets());
        let mut mac_addr = [0u8; 6];
        let mut mac_len: u32 = 6;

        let result = unsafe { SendARP(dest_ip, 0, mac_addr.as_mut_ptr().cast(), &mut mac_len) };
        if result == 0 && mac_len as usize >= 6 && mac_addr != [0u8; 6] {
            return Some(format!(
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                mac_addr[0], mac_addr[1], mac_addr[2],
                mac_addr[3], mac_addr[4], mac_addr[5]
            ));
        }
    }
    None
}

// ─── Linux: SIOCGARP ioctl ─────────────────────────────────────────

/// Linux: query kernel ARP table via `SIOCGARP` ioctl.
///
/// **Root cause of previous failures**: the kernel requires `arp_dev` to
/// identify the network device.  With an empty `arp_dev`, `dev` stays
/// NULL and [`neigh_lookup(&arp_tbl, &ip, NULL)`][kernel] cannot find
/// any entry because neighbour entries are indexed per-device.
///
/// The fix:
/// 1. Enumerate interfaces via `getifaddrs()`
/// 2. Find the interface whose subnet contains `ip`
/// 3. Set `arp_dev` to that interface name before calling `SIOCGARP`
///
/// [kernel]: https://github.com/torvalds/linux/blob/v6.6/net/ipv4/arp.c#L1118-L1138
#[cfg(target_os = "linux")]
fn get_mac_via_ioctl(ip: Ipv4Addr) -> Option<String> {
    use std::mem;

    const SIOCGARP: libc::c_ulong = 0x8954;
    const ATF_COM: libc::c_int = 0x02;
    const IFNAMSIZ: usize = 16;

    let if_name = find_interface_for_ip(ip)?;

    // ── Build `struct arpreq` ──────────────────────────────────────
    //
    // Layout (matches C `struct arpreq` from `<linux/if_arp.h>`):
    //
    //   ┌──────────────────────────────┬────────┬──────┐
    //   │  field                       │ offset │ size │
    //   ├──────────────────────────────┼────────┼──────┤
    //   │ arp_pa (sockaddr)            │   0    │  16  │
    //   │  ├ sa_family (u16)           │   0    │   2  │
    //   │  ├ sin_port (padding)        │   2    │   2  │
    //   │  └ sin_addr.s_addr (u32)     │   4    │   4  │
    //   │ arp_ha (sockaddr)            │  16    │  16  │
    //   │  ├ sa_family                 │  16    │   2  │
    //   │  └ sa_data (MAC)             │  18    │   6  │
    //   │ arp_flags (c_int)            │  32    │   4  │
    //   │ arp_netmask (sockaddr)       │  36    │  16  │
    //   │ arp_dev (char[16])           │  52    │  16  │
    //   └──────────────────────────────┴────────┴──────┘
    #[repr(C)]
    struct arpreq {
        arp_pa_family: u16,
        arp_pa_data: [u8; 14],
        arp_ha_family: u16,
        arp_ha_data: [u8; 14],
        arp_flags: libc::c_int,
        arp_netmask_family: u16,
        arp_netmask_data: [u8; 14],
        arp_dev: [libc::c_char; IFNAMSIZ],
    }

    let octets = ip.octets();

    unsafe {
        let sock = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
        if sock < 0 {
            return None;
        }

        let mut req: arpreq = mem::zeroed();
        req.arp_pa_family = libc::AF_INET as u16;
        req.arp_pa_data[2] = octets[0];
        req.arp_pa_data[3] = octets[1];
        req.arp_pa_data[4] = octets[2];
        req.arp_pa_data[5] = octets[3];

        let name_bytes = if_name.as_bytes();
        let len = name_bytes.len().min(IFNAMSIZ - 1);
        // SAFETY: arp_dev is a fixed-size C char array.
        let dev_slice = std::slice::from_raw_parts_mut(
            req.arp_dev.as_mut_ptr() as *mut u8,
            IFNAMSIZ,
        );
        dev_slice[..len].copy_from_slice(&name_bytes[..len]);
        dev_slice[len] = 0;

        let ret = libc::ioctl(sock, SIOCGARP, &mut req as *mut _);
        libc::close(sock);

        if ret == 0 && (req.arp_flags & ATF_COM) != 0 {
            let mac = &req.arp_ha_data[..6];
            if mac.iter().any(|&b| b != 0) {
                return Some(format!(
                    "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5],
                ));
            }
        }
    }
    None
}

/// Find the non-loopback network interface whose subnet contains `ip`.
///
/// Uses `getifaddrs()` to enumerate all AF_INET interfaces and checks
/// `(interface_ip & netmask) == (target_ip & netmask)`.
#[cfg(target_os = "linux")]
fn find_interface_for_ip(ip: Ipv4Addr) -> Option<String> {
    let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();

    let ip_nbo = u32::from(ip);

    if unsafe { libc::getifaddrs(&mut addrs) } != 0 || addrs.is_null() {
        return None;
    }

    let mut result: Option<String> = None;
    let mut walker = addrs;

    while !walker.is_null() {
        let ifa = unsafe { &*walker };

        if !ifa.ifa_addr.is_null()
            && !ifa.ifa_netmask.is_null()
            && unsafe { (*ifa.ifa_addr).sa_family } == libc::AF_INET as libc::sa_family_t
        {
            let addr = unsafe { &*(ifa.ifa_addr as *const libc::sockaddr_in) };
            let mask = unsafe { &*(ifa.ifa_netmask as *const libc::sockaddr_in) };

            let ifa_ip_nbo = u32::from_be(addr.sin_addr.s_addr);
            let ifa_mask_nbo = u32::from_be(mask.sin_addr.s_addr);

            if (ifa_ip_nbo & ifa_mask_nbo) == (ip_nbo & ifa_mask_nbo) {
                let name = unsafe { std::ffi::CStr::from_ptr(ifa.ifa_name) }
                    .to_string_lossy()
                    .into_owned();

                if name != "lo" {
                    result = Some(name);
                    break;
                }
            }
        }

        walker = unsafe { (*walker).ifa_next };
    }

    unsafe { libc::freeifaddrs(addrs) };
    result
}

/// Build a table of IP → MAC for all local non-loopback interfaces.
///
/// Enumerates interfaces via `getifaddrs()` and reads the MAC from
/// `/sys/class/net/<name>/address`.
#[cfg(target_os = "linux")]
fn get_local_ip_mac_table() -> HashMap<Ipv4Addr, String> {
    let mut table = HashMap::new();
    let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();

    if unsafe { libc::getifaddrs(&mut addrs) } != 0 || addrs.is_null() {
        return table;
    }

    let mut walker = addrs;
    while !walker.is_null() {
        let ifa = unsafe { &*walker };
        if !ifa.ifa_addr.is_null()
            && unsafe { (*ifa.ifa_addr).sa_family } == libc::AF_INET as libc::sa_family_t
        {
            let name = unsafe { std::ffi::CStr::from_ptr(ifa.ifa_name) }
                .to_string_lossy()
                .into_owned();

            if name != "lo" {
                let addr = unsafe { &*(ifa.ifa_addr as *const libc::sockaddr_in) };
                let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));

                let mac = std::fs::read_to_string(format!("/sys/class/net/{}/address", name))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .filter(|m| !m.is_empty() && m != "00:00:00:00:00:00");

                if let Some(m) = mac {
                    table.insert(ip, m);
                }
            }
        }
        walker = unsafe { (*walker).ifa_next };
    }

    unsafe { libc::freeifaddrs(addrs) };
    table
}


