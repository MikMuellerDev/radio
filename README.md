# Radio

> [!IMPORTANT]
> I am open to fix any bugs or introduce small features, but I do not have time to implement support for additional audio formats or other large features.

A headless music player which supports internet radio and streaming.


## Getting Started


- Get the latest release from
  [github.com/MikMuellerDev/radio/releases](https://github.com/MikMuellerDev/radio/releases)
- Extract the tar archive


### Installation


- Make sure that `sudo` is installed on the target machine
- Execute the file `install.sh` as non-root
- The application files will be installed at `/usr/bin/radio`
- The configuration file will be installed at `/etc/radio/config.toml`
- Radio can then be controlled via `systemd`:
  `sudo systemctl enable radio --now`


### Without Installation


- Execute the following command to start radio `./radio run`
- On the first launch, a new configuration file will be created without starting
  the service


## Configuration


Under normal conditions, the service creates a configuration file named
`config.toml` in the working directory where it is launched. If the service is
launched with the `-c` flag, the configuration path can be customized.


### Server Configuration


```toml
port = 8083
session_key = "must be over 64 characters long"
```


The configuration file includes the `port` variable which determines the port on
which radio's HTTP server will listen on.


The `session_key` must be over 64 characters long and should be kept secret. On
Unix-like systems, such a key can be generated using the following command:


```bash
openssl rand -hex 64
```


### Adding Users


Users can be added in the `users` list in the configuration file. Every user
must have a unique username and should be protected via a secure password.


```toml
[[users]]
username = "admin" # A unique username
password = "secret" # A secure password


[[users]]
username = "another_username"
password = "secret2"
```


### Adding Radio Stations / Audio Streams


Generally speaking, radio should support most **MP3** network streams. It does
