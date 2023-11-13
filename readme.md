# Who are you? DNS Client

This is a dns client written in rust in order to understand the dns protocol better.

Output
```
┌Header──────────────────────────────────────────────────────────────────┐
│OPCODE: Query, STATUS: NoError id: 9017                                 │
│qr, rd, ra,                                                             │
│QUERY: 1, ANSWERS: 5, AUTHORITY: 0, ADDITIONAL: 0                       │
└────────────────────────────────────────────────────────────────────────┘
┌Message─────────────────────────────────────────────────────────────────┐
│blog.toerktumlare.com    A                        IN                    │
└────────────────────────────────────────────────────────────────────────┘
┌Records─────────────────────────────────────────────────────────────────┐
│blog.toerktumlare.com    CNAME                    IN                    │
│tandolf.github.io        A                        IN                    │
│tandolf.github.io        A                        IN                    │
│tandolf.github.io        A                        IN                    │
│tandolf.github.io        A                        IN                    │
└────────────────────────────────────────────────────────────────────────┘
```

### Current supported record types
- A
- CNAME
- TXT

TODO:
- [ ] fancier formatting in the header section
- [ ] statistics section at the bottom
- [ ] print raw output `--raw`
- [ ] implement more record types (soa, null, ns, ptr, hinfo)
- [ ] implement input validation.
- [ ] implement ipv6 dns record types `rfc 3596` 
- [ ] add txt and cname flag to do such requests.
- [ ] formatting for txt and cname requests.
- [ ] att ttl and rdata to records
