fn main() {
    let libraries = [
        pkg_config::Config::new().atleast_version("54.1").probe("libavutil").unwrap(),
        pkg_config::Config::new().atleast_version("56.0").probe("libavcodec").unwrap(),
        pkg_config::Config::new().atleast_version("57.5").probe("libavformat").unwrap(),
    ];
    let mut wrapper = cc::Build::new();

    for lib in &libraries {
        // Pass include paths on to gcc. It'd be nice if pkg-config allowed fetching CFLAGS and
        // passing that on; see <https://github.com/alexcrichton/pkg-config-rs/issues/43>. But
        // the include paths are likely all that's included/significant for compilation.
        for p in &lib.include_paths {
            wrapper.include(p);
        }
    }

    wrapper.file("wrapper.c").compile("libwrapper.a");
}
