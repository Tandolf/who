# Who are you? DNS Client

This is a dns client written in rust in order to understand the dns protocol better.

Output
```
;; ========= HEADER ==========
;; OPCODE: Query, STATUS: NoError id: 2
;; qr, rd, ra,
;; QUERY: 1, ANSWERS: 5, AUTHORITY: 0, ADDITIONAL: 0

;; ======== Question =========
;blog.toerktumlare.com			IN	A

;; ========= Records =========
;blog.toerktumlare.com			1476	IN	CNAME	tandolf.github.io
;tandolf.github.io			1476	IN	A	185.199.109.153
;tandolf.github.io			1476	IN	A	185.199.111.153
;tandolf.github.io			1476	IN	A	185.199.110.153
;tandolf.github.io			1476	IN	A	185.199.108.153
```

### Current supported record types
- A
- CNAME
- TXT
