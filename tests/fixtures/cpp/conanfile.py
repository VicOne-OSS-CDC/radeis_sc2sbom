from conan import ConanFile
from conan.tools.cmake import cmake_layout

class MyProjectConan(ConanFile):
    name = "myproject"
    version = "1.0.0"

    # Dependency declarations (list format)
    requires = ["zlib/1.2.13", "openssl/3.1.2"]
    build_requires = ["cmake/3.27.0"]
    tool_requires = ["ninja/1.11.1"]
    test_requires = ["gtest/1.14.0"]

    settings = "os", "compiler", "build_type", "arch"

    def requirements(self):
        # Additional requirements via method calls
        self.requires("boost/1.82.0")

    def build_requirements(self):
        # Additional build requirements
        self.build_requires("doxygen/1.9.8")

    def layout(self):
        cmake_layout(self)

    def generate(self):
        pass

    def build(self):
        pass

    def package(self):
        pass
