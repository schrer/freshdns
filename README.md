# FreshDNS

A tool to update A records in a Cloudflare controlled domain, based on the current WAN IP address displayed on a FreshTomato router.  
It compares the current WAN IP with the available A records in the zone, then updates the entries if they do not match the current WAN IP.

## Config

The config file has to be named `config.toml` and needs to be saved in the same directory as the executable.
All values are mandatory.  
The Cloudflare API key and zone ID can be created or found through your Cloudflare dashboard.

``` toml
[freshtomato]
username = "user"
password = "password"
url = "<router_hostname>"

[cloudflare]
api_key="<cloudflare_api_key>"
zone_id="<zone_id>"
```