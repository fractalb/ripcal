# ripcal
Convert IP addresses dotted quads to/from integers
Also, Converts IP subnets to/from IP ranges

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

        ripcal <ip-addr/subnet> | "<ip-start - ip-end>"
                ip-addr/subnet will be converted to the corresponding
                ip-range ("start - end"). "start - end" (ip-range)
                will be converted to the minimal ip-addr/subnet which
                covers the given range.

        ripcal <ip-addr/subnet> | "<ip-start - ip-end>"
                ip-addr/subnet will be converted to ip-range ("start - end").
                "start_ip - end_ip" (ip-range) will be converted to minimal
                ip-addr/subnet which covers the given range.

        ripcal -m <ip-addr/subnet> | "<ip-start - ip-end>"
                Merge overlapping subnets and/or ip-ranges and print a minimal
                set of ip-ranges (and subnets) that cover all the input
                ip-ranges (and/or) subnets.

        ripcal -h or ripcal --help
                displays this help

        ripcal --version
                displays the program version\n"

Converts each <ip-address>:
- from dotted quad to hexadecimal integer
- from hexadecimal/decimal integers to dotted quad

Converts ip-address/subnet representation to/from ip-address range

Examples:

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

        $ ripcal 64420102 0x64420102
        64420102 = 3.214.249.6
        0x64420102 = 100.66.1.2

        $ ripcal 0xa141e28 a141e28
        0xa141e28 = 10.20.30.40
        a141e28 = 10.20.30.40

        $ ripcal 192.168.1.0/24
        192.168.1.0/24 = 192.168.1.0/24
        192.168.1.0/24 = 192.168.1.0 - 192.168.1.255

        $ ripcal "192.168.1.1 - 192.168.1.127"
        192.168.1.1 - 192.168.1.127 = 192.168.1.0/25
        192.168.1.0/25 = 192.168.1.0 - 192.168.1.127

Note1: "192.168.1.1 - 192.168.1.127" is converted to "192.168.1.0 - 192.168.1.127"
since the given input cannot be represented as an exact subnet. So, the input is
converted into minimal enclosing subnet (192.168.1.0/25)

Note2: The quotes around the IP address range ("192.168.1.1 - 192.168.1.127")
is needed because there is a space in between the ip-addresses. Alternatively,
192.168.1.1-192.168.1.127, with no spaces, will work without quotes.

        $ ripcal -m "192.168.1.1 - 192.168.1.127"
        [192.168.1.1 - 192.168.1.127]
        [192.168.1.1/32, 192.168.1.2/31, 192.168.1.4/30, 192.168.1.8/29, 192.168.1.16/28, 192.168.1.32/27, 192.168.1.64/26]

        $ ripcal -m "192.168.2.3 - 192.168.2.255" 192.168.3.0/24 192.168.2.0-192.168.2.2
        [192.168.2.0 - 192.168.3.255]
        [192.168.2.0/23]

        $ ripcal -m "192.168.1.0 - 192.168.1.255" "192.168.3.0 - 192.168.3.255"
        [192.168.1.0 - 192.168.1.255, 192.168.3.0 - 192.168.3.255]
        [192.168.1.0/24, 192.168.3.0/24]


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

