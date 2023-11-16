# Who are you? DNS Client

This is a dns client written in rust in order to understand the dns protocol better.

Output
```
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

TODO:
- [x] add ttl and rdata to records
- [x] implement input validation.
- [x] statistics section at the bottom
- [x] dynamically update size of gui blocks in accordance to data recvd
- [ ] add txt and cname flag to do such requests.
- [ ] fancier formatting in the header section
- [ ] print raw output `--raw`
- [ ] implement more record types (soa, null, ns, ptr, hinfo)
- [ ] implement ipv6 dns record types `rfc 3596` 
- [ ] formatting for txt and cname requests.
