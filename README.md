# FreshDNS

This is a tool meant to provide a dynamic DNS setup for users of a FreshTomato router and Cloudflare domains.  
The program runs as a one-shot application, so it needs to be coupled with e.g. systemd to regularly check and update the A records.

The application takes the current WAN IP from the routers admin page, compares it with all A records in the Cloudflare domains zone, then updates the entries if they do not match the current WAN IP. All A records get the same IP address.

## Config

The config file has to be named `config.toml` and needs to be saved in the working directory that you are using.
All values are mandatory.

``` toml
[freshtomato]
username = "user"
password = "password"
url = "<router_hostname>"

[cloudflare]
api_key="<cloudflare_api_key>"
zone_id="<zone_id>"
```

The Cloudflare API key needs to be created with rights to read and write the zone/domain you want to keep updated. The zone ID can be found through your Cloudflare dashboard.