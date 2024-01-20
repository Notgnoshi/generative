fn main() {
    // Options
    let enable_cmake_build = if cfg!(feature = "geom2graph-bindings") {
        String::from("ON")
    } else {
        std::env::var("GENERATIVE_CARGO_ENABLE_CMAKE_BUILD").unwrap_or_else(|_| "ON".to_string())
    };
    let enable_doxygen = std::env::var("GENERATIVE_CARGO_ENABLE_CMAKE_DOXYGEN")
        .unwrap_or_else(|_| "OFF".to_string());
    let enable_pch =
        std::env::var("GENERATIVE_CARGO_ENABLE_CMAKE_PCH").unwrap_or_else(|_| "OFF".to_string());
    let enable_lto =
        std::env::var("GENERATIVE_CARGO_ENABLE_CMAKE_LTO").unwrap_or_else(|_| "OFF".to_string());
    let enable_tests =
        std::env::var("GENERATIVE_CARGO_ENABLE_CMAKE_TESTS").unwrap_or_else(|_| "ON".to_string());
    if enable_cmake_build.is_empty()
        || enable_cmake_build == "OFF"
        || enable_cmake_build == "NO"
        || enable_cmake_build == "FALSE"
    {
        return;
    }

    // Rebuild if any of the C++ code changes
    println!("cargo:rerun-if-changed=tools/geom2graph.cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");

    for allow_dir in ["generative", "tests", "tools"] {
        for cmakelist in glob::glob(format!("{allow_dir}/**/CMakeLists.txt").as_str()).unwrap() {
            println!("cargo:rerun-if-changed={}", cmakelist.unwrap().display());
        }
        for cpp_source in glob::glob(format!("{allow_dir}/**/*.cpp").as_str()).unwrap() {
            println!("cargo:rerun-if-changed={}", cpp_source.unwrap().display());
        }
        for cpp_header in glob::glob(format!("{allow_dir}/**/*.h").as_str()).unwrap() {
            println!("cargo:rerun-if-changed={}", cpp_header.unwrap().display());
        }
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let install_dir = cmake::Config::new(".")
        .define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("CMAKE_GENERATOR", "Ninja")
        .define("GENERATIVE_BUILD_DOCS", enable_doxygen)
        .define("GENERATIVE_ENABLE_PCH", enable_pch)
        .define("GENERATIVE_ENABLE_LTO", enable_lto)
        .define("GENERATIVE_ENABLE_TESTING", &enable_tests)
        .define("GENERATIVE_TOOL_INSTALL_RPATH", "$ORIGIN/lib") // binaries just stashed in /target/debug/
        .build();

    // Copy ./target/debug/build/generative-<hash>/out/lib/ -> ./target/debug/lib/
    let src = format!("{}/lib", install_dir.display());
    let dest = format!("{}/../../../", &out_dir);
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    fs_extra::dir::copy(src, dest, &options).unwrap();

    let geom2graph = format!("{}/bin/geom2graph", install_dir.display());
    let dest = format!("{}/../../../geom2graph", &out_dir);
    std::fs::copy(geom2graph, dest).unwrap();

    let libgenerative = format!("{}/build/generative/libgenerative.a", install_dir.display());
    let dest = format!("{}/../../../lib/libgenerative.a", &out_dir);
    std::fs::copy(libgenerative, dest).unwrap();

    if enable_tests == "ON" || enable_tests == "YES" || enable_tests == "TRUE" {
        let tests = format!("{}/build/tests/tests", install_dir.display());
        let dest = format!("{}/../../../cxx-tests", &out_dir);
        std::fs::copy(tests, dest).unwrap();
    }

    let database = format!("{}/build/compile_commands.json", install_dir.display());
    let dest = format!("{manifest_dir}/compile_commands.json");
    std::fs::copy(database, dest).unwrap();

    #[cfg(feature = "geom2graph-bindings")]
    {
        println!("cargo-rust-c-link-search=native={}/../../../lib/", &out_dir);
        println!("cargo:rust-c-link-lib=static=generative");
        println!("cargo:rust-c-link-lib=geos");

        let cxxbridge_sources = ["generative/cxxbridge/geometry_collection.rs"];
        cxx_build::bridges(cxxbridge_sources).compile("cxxbridge");
    }
}
