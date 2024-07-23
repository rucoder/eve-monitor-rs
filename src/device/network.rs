use std::net::IpAddr;

use crate::ipc::eve_types::NetworkPortStatus;
// use macaddr::MacAddr;
use serde_json::json;

#[derive(Debug)]
pub struct NetworkInterface {
    name: String,
    is_mngmt: bool,
    addresses: Vec<IpAddr>,
    default_gateway: Option<Vec<IpAddr>>,
    // mac: MacAddr,
}

impl From<NetworkPortStatus> for NetworkInterface {
    fn from(port: NetworkPortStatus) -> Self {
        // parse address list
        let addresses = port.addr_info_list.iter().map(|addr| addr.addr).collect();

        NetworkInterface {
            name: port.if_name,
            addresses,
            is_mngmt: port.is_mgmt,
            default_gateway: port.default_routers,
            // mac: MacAddr::from(port.mac_addr),
        }
    }
}

#[derive(Debug)]
pub enum IoError {
    NetworkListError,
}

mod tests {
    use serde_json::from_value;

    use super::*;
    fn get_network_port_status() -> NetworkPortStatus {
        let json = json!({
            "IfName": "eth1",
            "Phylabel": "eth1",
            "Logicallabel": "eth1",
            "Alias": "",
            "IsMgmt": true,
            "IsL3Port": true,
            "Cost": 0,
            "Dhcp": 4,
            "Type": 0,
            "Subnet": {
                "IP": "192.168.2.0",
                "Mask": "////AA=="
            },
            "NtpServer": "",
            "DomainName": "",
            "DNSServers": [
                "192.168.2.3"
            ],
            "NtpServers": null,
            "AddrInfoList": [
                {
                    "Addr": "192.168.2.10",
                    "Geo": {
                        "ip": "",
                        "hostname": "",
                        "city": "",
                        "region": "",
                        "country": "",
                        "loc": "",
                        "org": "",
                        "postal": ""
                    },
                    "LastGeoTimestamp": "0001-01-01T00:00:00Z"
                },
                {
                    "Addr": "fec0::21b8:b579:8b9c:3cda",
                    "Geo": {
                        "ip": "",
                        "hostname": "",
                        "city": "",
                        "region": "",
                        "country": "",
                        "loc": "",
                        "org": "",
                        "postal": ""
                    },
                    "LastGeoTimestamp": "0001-01-01T00:00:00Z"
                },
                {
                    "Addr": "fe80::6f27:5660:de21:d553",
                    "Geo": {
                        "ip": "",
                        "hostname": "",
                        "city": "",
                        "region": "",
                        "country": "",
                        "loc": "",
                        "org": "",
                        "postal": ""
                    },
                    "LastGeoTimestamp": "0001-01-01T00:00:00Z"
                }
            ],
            "Up": true,
            "MacAddr": "UlQAEjRX",
            "DefaultRouters": [
                "192.168.2.2",
                "fe80::2"
            ],
            "MTU": 1500,
            "WirelessCfg": {
                "WType": 0,
                "CellularV2": {
                    "AccessPoints": null,
                    "Probe": {
                        "Disable": false,
                        "Address": ""
                    },
                    "LocationTracking": false
                },
                "Wifi": null,
                "Cellular": null
            },
            "WirelessStatus": {
                "WType": 0,
                "Cellular": {
                    "LogicalLabel": "",
                    "PhysAddrs": {
                        "Interface": "",
                        "USB": "",
                        "PCI": "",
                        "Dev": ""
                    },
                    "Module": {
                        "Name": "",
                        "IMEI": "",
                        "Model": "",
                        "Manufacturer": "",
                        "Revision": "",
                        "ControlProtocol": "",
                        "OpMode": ""
                    },
                    "SimCards": null,
                    "ConfigError": "",
                    "ProbeError": "",
                    "CurrentProvider": {
                        "PLMN": "",
                        "Description": "",
                        "CurrentServing": false,
                        "Roaming": false,
                        "Forbidden": false
                    },
                    "VisibleProviders": null,
                    "CurrentRATs": null,
                    "ConnectedAt": 0,
                    "IPSettings": {
                        "Address": null,
                        "Gateway": "",
                        "DNSServers": null,
                        "MTU": 0
                    },
                    "LocationTracking": false
                }
            },
            "Proxies": null,
            "Exceptions": "",
            "Pacfile": "",
            "NetworkProxyEnable": false,
            "NetworkProxyURL": "",
            "WpadURL": "",
            "pubsub-large-ProxyCertPEM": null,
            "L2Type": 0,
            "VLAN": {
                "ParentPort": "",
                "ID": 0
            },
            "Bond": {
                "AggregatedPorts": null,
                "Mode": 0,
                "LacpRate": 0,
                "MIIMonitor": {
                    "Enabled": false,
                    "Interval": 0,
                    "UpDelay": 0,
                    "DownDelay": 0
                },
                "ARPMonitor": {
                    "Enabled": false,
                    "Interval": 0,
                    "IPTargets": null
                }
            },
            "LastFailed": "2024-07-22T06:27:12.052879635Z",
            "LastSucceeded": "0001-01-01T00:00:00Z",
            "LastError": "All attempts to connect to https://zedcloud.alpha.zededa.net/api/v2/edgedevice/ping failed: interface eth1: no DNS server available"
        });

        from_value(json).unwrap()
    }

    #[test]
    fn test_from() {
        let port = get_network_port_status();
        let network_interface = NetworkInterface::from(port);
        println!("{:?}", network_interface);
        assert_eq!(network_interface.name, "eth1");
    }
}
