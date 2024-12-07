# Porkbun / Dynamic DNS bridge

A Fastly Compute app that bridges the Porkbun API with a dynamic DNS server that can be used with Unifi routers (though it can be used for anything else too).

## Deploy

`fastly compute deploy`

## Fastly Service Configuration

You will need a backend named `porkbun` pointing to `api.porkbun.com` on port 443.

You will need a Config Store named `porkbun_config` with a single `domain` key that points to the domain you will be configuring.

You will need a Secret Store named `porkbun_secrets` which contains
three items:

- `api_key` - your Porkbun API key
- `secret_api_key` - your Porkbun secret API key
- `auth_token` - an arbitrarily chosen string that the client must provide to authenticate the request

Resource link these stores to the service you create.

## Unifi Configuration

If you're using this with a Unifi system, you'll need to configure the Dynamic DNS settings in the controller.  Click on "Internet", choose your WAN connection, then "Create New Dynamic DNS".

- Service: `custom`
- Hostname: The hostname you want to update, eg. `home.example.com`
- Username: Anything, this isn't used.
- Password: Anything, this isn't used.
- Server: `<service hostname>/nic/update?hostname=%h&myip=%i&auth_token=<auth_token>`

The service hostname is provided when you first deploy the Fastly service.  If you're fancy you'll put it on your own domain, but whatever `edgecompute.app` hostname they give you is fine too.  The auth token is the same one you put into the secret store.
