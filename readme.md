# Who are you? DNS Client

This is a dns client written in rust in order to understand the dns protocol better.

Output
```
┌Header──────────────────────────────────────────────────────────────────┐
│OPCODE: Query, STATUS: NoError id: 27995                                │
│qr, rd, ra,                                                             │
│QUERY: 1, ANSWERS: 5, AUTHORITY: 0, ADDITIONAL: 0                       │
└────────────────────────────────────────────────────────────────────────┘
┌Message─────────────────────────────────────────────────────────────────┐
│blog.toerktumlare.com             IN       A                            │
└────────────────────────────────────────────────────────────────────────┘
┌Records─────────────────────────────────────────────────────────────────┐
│blog.toerktumlare.com     3469    IN       CNAME   tandolf.github.io    │
│tandolf.github.io         3469    IN       A       185.199.108.153      │
│tandolf.github.io         3469    IN       A       185.199.110.153      │
│tandolf.github.io         3469    IN       A       185.199.111.153      │
│tandolf.github.io         3469    IN       A       185.199.109.153      │
└────────────────────────────────────────────────────────────────────────┘
```

### Current supported record types
- A
- CNAME
- TXT

TODO:
- [x] add ttl and rdata to records
- [ ] implement input validation.
- [ ] statistics section at the bottom
- [ ] add txt and cname flag to do such requests.
- [ ] fancier formatting in the header section
- [ ] print raw output `--raw`
- [ ] implement more record types (soa, null, ns, ptr, hinfo)
- [ ] implement ipv6 dns record types `rfc 3596` 
- [ ] formatting for txt and cname requests.
