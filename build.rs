fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
    cc::Build::new().file("src/logger.c").compile("liblogger.a");
}
