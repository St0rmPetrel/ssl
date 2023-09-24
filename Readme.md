# ssl

A learning project whose main goal was to understand the operation of the `md5` and `sha256` hashing algorithms.

There are documentations of implementation of these algorithms in this project:
- [md5](doc/md5.md)
- [sha256](doc/sha256.md)


## Usage/Examples
```sh
ssl --help
```
```
Usage: ssl <COMMAND>

Commands:
  md5     compute and check MD5 message digest
  sha256  compute and check SHA256 message digest
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

#### MD5 command
```sh
ssl md5 --help
```
```
compute and check MD5 message digest

Usage: ssl md5 [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Files to digest (optional; default is stdin). With no FILE, or when FILE is -, read standard input

Options:
  -t, --tag    create a BSD-style checksum if true. else create GNU style checksum file
  -c, --check  read checksums from the FILEs and check them
  -h, --help   Print help
```

##### Examples
```sh
echo "Hello World" | ssl md5
```
```
e59ff97941044f85df5297e1c302d260  -
```

```sh
echo "Hello World" > hello
ssl md5 hello
```
```
e59ff97941044f85df5297e1c302d260  hello
```

```sh
ssl md5 hello > hello.sum
ssl md5 -c hello.sum
```
```
hello: OK
```

#### SHA256 command
```sh
ssl sha256 --help
```
```
compute and check SHA256 message digest

Usage: ssl sha256 [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Files to digest (optional; default is stdin). With no FILE, or when FILE is -, read standard input

Options:
  -t, --tag    create a BSD-style checksum if true. else create GNU style checksum file
  -c, --check  read checksums from the FILEs and check them
  -h, --help   Print help
```

##### Examples
```sh
echo "Hello World" | ssl sha256
```
```
d2a84f4b8b650937ec8f73cd8be2c74add5a911ba64df27458ed8229da804a26  -
```

```sh
echo "Hello World" > hello
ssl sha256 hello
```
```
d2a84f4b8b650937ec8f73cd8be2c74add5a911ba64df27458ed8229da804a26  hello
```

```sh
ssl sha256 hello > hello.sum
ssl sha256 -c hello.sum
```
```
hello: OK
```
