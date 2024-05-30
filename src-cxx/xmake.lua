add_rules("mode.debug", "mode.release")

add_requires("cmake::OpenCV")

target("phash_test")
    set_kind("binary")
    add_files("src/phash_test.cpp")
    add_packages("cmake::OpenCV")