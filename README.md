# ripcal
Convert IP addresses dotted quads to/from integers

        ripcal [-i | -x | -q ] <ip-address>...
                Converts each <ip-address> to different formats
                If no option is provided then ip-quads will be
                converted to hexa-decimal integers and all
                integers will be converted to ipv4 dotted quads.

        Options:
                --integer or -i
                        Converts to a deca-decimal integer
                --hex or -x
                        Converts to a hexa-decimal integer
                --ipv4 or -q
                        Converts to an ip-quad
                --reverse-bytes or -r
                        Reverses the byte order

        ripcal --version
                displays the program version\n"

        ripcal -h or ripcal --help
                displays this help

Converts each <ip-address>:
- from dotted quad to hexadecimal integer
- from hexadecimal/decimal integers to dotted quad

example:

    $ ./ripcal 192.168.2.4 0xc0a1b203 2886732292
    192.168.2.4 = 0xc0a80204
    0xc0a1b203 = 192.161.178.3
    2886732292 = 172.16.10.4


