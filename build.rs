fn main() {
    pkg_config::Config::new()
        .atleast_version("1.4.99.1")
        .probe("x11")
        .unwrap();
    pkg_config::Config::new()
        .atleast_version("3.1.0")
        .probe("xfixes")
        .unwrap();
}
