#[cfg(not(feature = "cxx"))]
fn main() {}

#[cfg(feature = "cxx")]
fn main() {
    #[rustfmt::skip]
    let enable_tests = if cfg!(feature = "cxx-tests") { "ON" } else { "OFF" };

    // Rebuild if any of the C++ code changes
    println!("cargo:rerun-if-changed=CMakeLists.txt");

    // TODO: Remove tools/ when geom2graph.rs is swapped in
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
        for cpp_header in glob::glob(format!("{allow_dir}/**/*.hpp").as_str()).unwrap() {
            println!("cargo:rerun-if-changed={}", cpp_header.unwrap().display());
        }
    }

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let install_dir = cmake::Config::new(".")
        .define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
        .define("CMAKE_INSTALL_LIBDIR", "lib")
        .define("CMAKE_GENERATOR", "Ninja")
        .define("GENERATIVE_BUILD_DOCS", "OFF")
        .define("GENERATIVE_ENABLE_PCH", "OFF")
        .define("GENERATIVE_ENABLE_LTO", "OFF")
        .define("GENERATIVE_ENABLE_TESTING", enable_tests)
        .define("GENERATIVE_TOOL_INSTALL_RPATH", "$ORIGIN/lib") // binaries just stashed in /target/debug/
        .build();

    // Copy ./target/debug/build/generative-<hash>/out/lib/ -> ./target/debug/lib/
    let src = format!("{}/lib", install_dir.display());
    let dest = format!("{}/../../../", &out_dir);
    let mut options = fs_extra::dir::CopyOptions::new();
    options.overwrite = true;
    fs_extra::dir::copy(src, dest, &options).unwrap();

    // TODO: Remove in favor of geom2graph.rs
    let geom2graph = format!("{}/bin/geom2graph-cxx", install_dir.display());
    let dest = format!("{}/../../../geom2graph-cxx", &out_dir);
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

    #[cfg(feature = "cxx-bindings")]
    {
        println!("cargo:rustc-link-search=native={}/../../../lib/", &out_dir);
        println!("cargo:rustc-link-lib=static=generative");
        println!("cargo:rustc-link-lib=log4cplus");
        println!("cargo:rustc-link-lib=geos");

        let cxxbridge_sources = [
            "generative/cxxbridge/coord_ffi.rs",
            "generative/cxxbridge/geometry_collection_ffi.rs",
            "generative/cxxbridge/geometry_graph_ffi.rs",
            "generative/cxxbridge/noder_ffi.rs",
        ];
        cxx_build::bridges(cxxbridge_sources)
            .include("generative/cxxbridge/")
            .include("generative/include/")
            .define("USE_UNSTABLE_GEOS_CPP_API", "") // silence the geos #warning about unstable API
            .flag("-isystemsubmodules/geos/include/")
            .std("c++17")
            .compile("cxxbridge");

        for src in cxxbridge_sources {
            println!("cargo:rerun-if-changed={src}");
        }
    }
}
