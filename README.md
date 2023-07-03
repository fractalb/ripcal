# ripcal
Convert IP address dotted quads to/from integers

    ripcal <ip-address>...

Converts each <ip-address>:
- from dotted quad to hexadecimal integer
- from hexadecimal/decimal integers to dotted quad

example:

    $ ./ripcal 192.168.2.4 0xc0a1b203 2886732292
    192.168.2.4 = 0xc0a80204
    0xc0a1b203 = 192.161.178.3
    2886732292 = 172.16.10.4


