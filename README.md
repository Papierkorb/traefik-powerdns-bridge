# Traefik PowerDNS Bridge

The Traefik PowerDNS Bridge is a simple HTTP bridge that facilitates domain lookup in a Traefik instance and responds to requests made from PowerDNS. This enables automatic configuration of necessary domain names in your infrastructure.

## Installation

Follow these steps to install and set up the Traefik PowerDNS Bridge:

1. Build using `cargo build --release`, or if you Docker do `docker run -it --rm -v $PWD:/app -w /app -u $UID:$GID rust:1.78 cargo build --release`
2. Copy `target/release/traefik-powerdns-bridge` and `traefik-powerdns-bridge.service` to your server
3. Make any modifications needed to `traefik-powerdns-bridge.service` (fix the path to the binary)
4. Move `traefik-powerdns-bridge.service to /etc/systemd/system/`
5. Run `sudo systemctl daemon-reload`, then `sudo systemctl enable --now traefik-powerdns-bridge.service`
6. Install the remote backend, on Ubuntu: `apt install pdns-backend-remote`
7. Add these lines (change as needed) to your /etc/powerdns/pdns-recursor.conf:
   ```
   zone-cache-refresh-interval=0
   remote-connection-string=http:url=http://127.0.0.1:8787/dns
   launch=remote
   ```
8. Restart PowerDNS: `sudo systemctl restart pdns-recursor.service`

Once installed and configured, Traefik PowerDNS Bridge will automatically respond to PowerDNS requests and perform domain lookups in your Traefik instance.

### Configuration

Configuration is done via environment variables:

* `TRAEFIK_IP`: Mandatory, IP address of the traefik host (Not host name!), like `192.168.1.123`
* `MY_ZONES`: Mandatory, domain names of your zones (Your primary domain), like `home.my-domain.com`. Comma separated.
* `TRAEFIK_API_PORT`: Optional, the API port if changed. Defaults to `8080`.
* `LISTEN`: Optional, defaults to `127.0.0.1:8787` (Change if you're not running it on the DNS host itself).

Add any combination of these into the systemd service unit like so:

```
Environment=TRAEFIK_IP=192.168.1.123 LISTEN=0.0.0.0:8787
```

## Contributing

Contributions are welcome! If you find any issues or have suggestions for improvements, feel free to open an issue or create a pull request.

## License

This project is licensed under the GPL license version 3.
