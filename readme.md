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
;blog.toerktumlare.com			IN	CNAME
;tandolf.github.io			    IN	A
;tandolf.github.io			    IN	A
;tandolf.github.io			    IN	A
;tandolf.github.io			    IN	A
```

### Current supported record types
- A
- CNAME
- TXT
