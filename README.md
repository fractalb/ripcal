# ripcal
Convert IP addresses dotted quads to/from integers

        ripcal [-i | -x | -q ] [-r] <ip-address>...
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

        $ ripcal 192.168.2.4 0xc0a80204 3232236036
        192.168.2.4 = 0xc0a80204
        0xc0a80204 = 192.168.2.4
        3232236036 = 192.168.2.4

        $ ripcal -q 192.168.2.4 0xc0a80204 3232236036
        192.168.2.4 = 192.168.2.4
        0xc0a80204 = 192.168.2.4
        3232236036 = 192.168.2.4

        $ ripcal -x 192.168.2.4 0xc0a80204 3232236036
        192.168.2.4 = 0xc0a80204
        0xc0a80204 = 0xc0a80204
        3232236036 = 0xc0a80204

        $ ripcal -i 192.168.2.4 0xc0a80204 3232236036
        192.168.2.4 = 3232236036
        0xc0a80204 = 3232236036
        3232236036 = 3232236036

        $ ripcal 0xc0a80204 -r 0xc0a80204
        0xc0a80204 = 192.168.2.4
        Reverse 0xc0a80204 = 4.2.168.192

When no ip-address arguments are given on the command, then the program
will read from stdin and write to stdout (filter mode).

        $ ripcal
        1.2.3.4
        0x1020304
        3232236036
        192.168.2.4
        ^D

        $ echo "192.168.2.3" | ripcal
        0xc0a80203

The command expects only one IP address per line in filter mode. The below
commands won't work as expected

        $ ripcal <<<"192.168.2.3 192.168.3.2"
        Invaid IP address: 192.168.2.3 192.168.3.2

        $ echo "192.168.2.3 192.168.3.2" | ripcal
        Invaid IP address: 192.168.2.3 192.168.3.2

