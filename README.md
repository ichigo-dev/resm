# Remote Server Control tools

A group of tools for simplifying the work done by connecting to a remote server
in the development of web systems.


## Set up

### Create Configuration files

First, create a configuration file to use this command according to the JSON
format below. (An example configuration file can be found in `env`)

Configuration files are automatically and recursively loaded from `$HOME/env`,
so they can be managed separately for each project, for example. It is also
possible to change the read path of these configuration files by specifying the
`-e` option common to all subcommands.

```json
[
    {
        "project": "project name",
        "environment": "dev",

        "remote_path": "path/to/remote/project/root",
        "remote_cache_path": "path/to/remote/project/cache",

        "git_path": "path/to/git/repo",
        "git_src_path": "path/to/git/src",
        "backup_path": "path/to/backup",

        "db_host": "db host name",
        "db_host_reader": "db host name for reader",
        "db_port": 3306,
        "db_name": "db name",
        "db_user": "db user",
        "db_password": "db password",
        "db_root_user": "db root user",
        "db_root_password": "db root password",

        "connect_info":
        {
            "host": "host name",
            "port": 22,
            "user": "user",
            "identity_file": "path/to/key_file"
        },

        "tunnels":
        [
            {
                "host": "bastion host name",
                "port": 22,
                "user": "user",
                "identity_file": "path/to/key_file"
            }
        ]
    }
]
```

The meaning of each field is as follows. Required fields are marked with an
asterisk (*).

- project (*): Optional project name
- environment (*): Optional environment name
- remote_path: Reference path to be operated in the destination server
               (absolute path)
- remote_cache_path: Path to the cache directory in the destination server
                     (absolute path)
- git_path: Path to the git repository (absolute path)
- git_src_path: Path to the source directory in the git repository
                (absolute path)
- backup_path: Path to the backup directory (absolute path)
- db_host: Host name of the database server
- db_host_reader: Host name of the database server for reader
- db_port: Port number of the database server
- db_name: Database name
- db_user: Database user name
- db_password: Database password
- db_root_user: Database root user name
- db_root_password: Database root password
- connect_info (*)
    - host (*): Host name or IP address to connect to
    - port: Port number of the server
    - user: User name
    - password: Password (Entering password cannot be omitted)
    - identity_file: Path to the identity file (absolute path)
- tunnels: Information on the step server to be passed through when
           connecting (array of connect_info)

### Initialize

Execute the following command to generate the SSH configuration file.

```sh
$ resm init
```

Place the generated config file in `$HOME/.ssh/config` or append appropriately.


## Commands

### list

Lists projects.

### show

Shows the project setting.

### replace

Replaces the project directory in the destination server with the local
project directory. In short, it is useful when you want to perform a full
update of an application.

### patch

Uploads only specified files in the local repository to the remote server.

### clear

Clears the remote cache directory.

### backup

Backs up the remote directory.

### backup-db

Backs up the database.


## Caution

This tool depends on packages that are only compatible with Unix, so if you are
developing in a Windows environment, install this command on WSL and use the
Windows files in `/mnt` as `git_path` or `git_src_path`. Be sure to specify a
path in your system.
