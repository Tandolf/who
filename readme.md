# Who are you? DNS Client

This is a dns client written in rust in order to understand the dns protocol better.

Output
```bash
$ who blog.toerktumlare.com
== Who are you? == v1.0.0 == blog.toerktumlare.com ==
┌Header──────────────────────────────────────────────────────────────────┐
│OPCODE: Query, STATUS: NoError id: 32419                                │
│qr, rd, ra,                                                             │
│QUERY: 1, ANSWERS: 5, AUTHORITY: 0, ADDITIONAL: 0                       │
└────────────────────────────────────────────────────────────────────────┘
┌Message─────────────────────────────────────────────────────────────────┐
│blog.toerktumlare.com             IN       A                            │
└────────────────────────────────────────────────────────────────────────┘
┌Records─────────────────────────────────────────────────────────────────┐
│blog.toerktumlare.com     2792    IN       CNAME   tandolf.github.io    │
│tandolf.github.io         2792    IN       A       185.199.110.153      │
│tandolf.github.io         2792    IN       A       185.199.108.153      │
│tandolf.github.io         2792    IN       A       185.199.111.153      │
│tandolf.github.io         2792    IN       A       185.199.109.153      │
└────────────────────────────────────────────────────────────────────────┘
┌Statistics──────────────────────────────────────────────────────────────┐
│Query time: 4 msec                                                      │
│When: 2023-11-15 18:11:43                                               │
│Msg SENT: 39 bytes                                                      │
│Msg RCVD: 134 bytes                                                     │
└────────────────────────────────────────────────────────────────────────┘
```

### Current supported record types
- A
- CNAME
- TXT
- AAAA (ipv6, [rfc 3596](https://datatracker.ietf.org/doc/html/rfc3596))
- NS

## help
There are support for several type of queries, all documented in the help menu.

```bash
$ who -h

== Who are you? == v0.1.0

Usage: who [OPTIONS] [DOMAIN] [COMMAND]

Commands:
  txt    fetch text records
  cname  fetch cname records
  a      fetch A (ipv4) records
  aaaa   fetch AAAA (ipv6) records
  ns     return NS (name server) records
  help   Print this message or the help of the given subcommand(s)

Arguments:
  [DOMAIN]  the domain you are asking for

Options:
  -r, --raw-records
  -h, --help         Print help (see more with '--help')
  -V, --version      Print version
```

## examples

standard query:
```
who www.google.com
```

query for a specific record:
```
who cname www.google.com
```

query ipv6 (AAAA)
```
who aaaa www.google.com
```

raw output
```
who --raw blog.toerktumlare.com

blog.toerktumlare.com			3600	IN	CNAME	tandolf.github.io
tandolf.github.io			3600	IN	A	185.199.111.153
tandolf.github.io			3600	IN	A	185.199.108.153
tandolf.github.io			3600	IN	A	185.199.109.153
tandolf.github.io			3600	IN	A	185.199.110.153
```

TODO:
- [ ] fancier formatting in the header section
- [ ] implement more record types (soa, null, ns, ptr, hinfo)
- [x] Fix help menu so it looks good
- [x] implement ipv6 dns record types `rfc 3596` 
- [x] print raw output `--raw`
- [x] add txt and cname flag to do such requests.
- [x] dynamically update size of gui blocks in accordance to data recvd
- [x] add ttl and rdata to records
- [x] implement input validation.
- [x] statistics section at the bottom
