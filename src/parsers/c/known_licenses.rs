pub fn lookup(lib_name: &str) -> Option<String> {
    let name = lib_name.to_lowercase();
    let license = match name.as_str() {
        "ssl" | "crypto" | "openssl"          => "Apache-2.0",
        "z" | "zlib"                           => "Zlib",
        "curl"                                 => "curl",
        "pthread" | "dl" | "rt"               => "LGPL-2.1-or-later",
        "m"                                    => "LGPL-2.1-or-later",
        "bpf" | "libbpf"                       => "LGPL-2.1-or-later",
        "protobuf"                             => "BSD-3-Clause",
        "mbedtls" | "mbedcrypto" | "mbedx509" => "Apache-2.0",
        "elfutils" | "elf" | "dw"             => "GPL-2.0-or-later",
        "glib-2.0" | "glib"                   => "LGPL-2.1-or-later",
        "xml2" | "libxml2"                    => "MIT",
        "sqlite3" | "sqlite"                  => "blessing",
        "pcre" | "pcre2"                      => "BSD-3-Clause",
        "uuid"                                => "BSD-3-Clause",
        "lzma" | "xz"                         => "LGPL-2.1-or-later",
        "bz2"                                 => "bzip2-1.0.5",
        "systemd" | "sd-daemon"               => "LGPL-2.1-or-later",
        "dbus-1"                              => "AFL-2.1",
        "expat"                               => "MIT",
        "ncurses" | "curses" | "tinfo"        => "MIT",
        "readline"                            => "GPL-3.0-or-later",
        "ffi"                                 => "MIT",
        "event" | "event_core"               => "BSD-3-Clause",
        _ => return None,
    };
    Some(license.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_ssl() {
        assert_eq!(lookup("ssl"), Some("Apache-2.0".to_string()));
    }

    #[test]
    fn test_lookup_zlib() {
        assert_eq!(lookup("z"), Some("Zlib".to_string()));
    }

    #[test]
    fn test_lookup_unknown() {
        assert_eq!(lookup("unknown_lib"), None);
    }

    #[test]
    fn test_lookup_case_insensitive() {
        assert_eq!(lookup("OpenSSL"), Some("Apache-2.0".to_string()));
    }
}
