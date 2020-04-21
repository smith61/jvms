# JVMS

JVMS manages multiple java toolchains similar to rustup manages rust toolchains. JVMS does not manage installing java toolchains, and those must be handled outside of this tool.

## Installing JVMS

JVMS can be installed from source using with the following commands

```shell
# Install JVMS binary and generate shims
cargo run --release -- install <installation_directory>

# Configure the default toolchain. The -f switch can be ommited on later toolchain additions.
jvms toolchain add -f <toolchain_name> <path_to_java_home>
jvms default <toolchain_name>
```

After the above is completed the shims provided in `<installation_directory>` will use the default toolchain unless overriden. To override the default toolchain for a directory the following command can be used.

```shell
jvms override set <toolchain_name>
```

## Supported shims

JVMS provides shims for the following java tools. If a shim is missing, feel free to file an issue or open a PR to add support for the shim.

* `jar`
* `java`
* `javac`
* `javadoc`
* `javah`
* `javap`
* `javaw`
