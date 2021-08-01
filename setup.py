from setuptools import setup
from setuptools_rust import RustExtension

setup(
    name="generative",
    version="0.1.0",
    rust_extensions=[RustExtension("generative.rust", path="generative/rust/Cargo.toml")],
    packages=["generative"],
    zip_safe=False,
)
