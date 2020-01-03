Cloudflare DDNS
===============

A command-line utility which updates the value of a Cloudflare DNS record to your public IP.


Usage
-----

```
$ cloudflare-ddns --help
cloudflare-ddns 0.1.0

USAGE:
    cloudflare-ddns --auth-token <auth-token> --record-name <record-name> --zone-id <zone-id>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Enable verbose logging

OPTIONS:
        --auth-token <auth-token>      API token generated on the "My Account" page
        --record-name <record-name>    DNS record "name" from domain "DNS" page
        --zone-id <zone-id>            Zone ID from domain "Overview" page, "API" section
```

Example:

```
$ cloudflare-ddns --auth-token XYZ789 --zone-id ABC123 --record-name server.example.com
Public IP: 12.34.56.78
Successfully updated server.example.com!
```


Install
-------

```
$ cargo install cloudflare-ddns
```



License
-------

    Copyright 2020 Jake Wharton

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
