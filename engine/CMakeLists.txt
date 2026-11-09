# borg-reborn-rs's build (docs/SPEC.md §3.6; task TE.4).
#
# Compiles a patched copy of the pristine upstream tree (engine/angband/,
# engine/patches/), our headless frontend (engine/frontend/) and the Rust
# Borg's staticlib into game/angband, with the game data in game/lib. The
# C Borg (src/borg/) is not built: the Rust Borg exports do_cmd_borg in its
# place. Upstream's own CMakeLists.txt is not used (it lists src/borg/
# inside its core library); its source list is copied in sources.cmake.
#
#   cmake -S engine -B build/rust [-DBORG_FEATURES=test-hallucinate] [-DSUPPORT_X11=ON]
#   make -C build/rust
#
# harness/build.sh is the entry point (the zoo calls it).
cmake_minimum_required(VERSION 3.13)
project(borg-reborn-engine C)

set(ENGINE "${CMAKE_CURRENT_SOURCE_DIR}")
set(REPO "${ENGINE}/..")
set(TREE "${CMAKE_BINARY_DIR}/tree")

option(SUPPORT_X11 "Also build the X11 frontend (to watch the Borg play)" OFF)
set(BORG_FEATURES "" CACHE STRING "Extra cargo features of the Rust Borg (tests only)")

# The patched copy of the tree, remade whenever the tree or a patch changes.
file(GLOB PATCHES "${ENGINE}/patches/*.patch")
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${PATCHES} "${ENGINE}/UPSTREAM")
file(REMOVE_RECURSE "${TREE}")
execute_process(COMMAND "${ENGINE}/apply-patches.sh" "${TREE}" RESULT_VARIABLE PATCHED)
if(NOT PATCHED EQUAL 0)
    message(FATAL_ERROR "engine/apply-patches.sh failed")
endif()

include("${ENGINE}/sources.cmake")
list(TRANSFORM ANGBAND_CORE_SOURCES PREPEND "${TREE}/")

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/game")
add_compile_options(-Wall -Wextra -pedantic -Wno-unused-parameter)

set(FRONTENDS "${TREE}/src/main.c" "${ENGINE}/frontend/main-borgh.c")
if(SUPPORT_X11)
    list(APPEND FRONTENDS "${TREE}/src/main-x11.c")
endif()
add_executable(angband ${ANGBAND_CORE_SOURCES} ${FRONTENDS})
set_target_properties(angband PROPERTIES C_STANDARD 99 C_EXTENSIONS ON)
target_include_directories(angband PRIVATE "${TREE}/src" "${ENGINE}/frontend")
# As upstream's self-contained build (SC_INSTALL): data in ./lib/ next to the
# executable. ALLOW_BORG as upstream's SUPPORT_BORG; BORG_HARNESS and
# USE_BORGH turn on our patch series and frontend.
target_compile_definitions(angband PRIVATE
    ALLOW_BORG BORG_HARNESS USE_BORGH
    DEFAULT_CONFIG_PATH="./lib/" DEFAULT_LIB_PATH="./lib/" DEFAULT_DATA_PATH="./lib/")
if(SUPPORT_X11)
    find_package(X11 REQUIRED)
    target_compile_definitions(angband PRIVATE USE_X11)
    target_include_directories(angband PRIVATE ${X11_INCLUDE_DIR})
    target_link_libraries(angband PRIVATE ${X11_LIBRARIES})
endif()

# The Rust Borg: a staticlib built by cargo, with the harness feature.
set(CARGO_TARGET "${CMAKE_BINARY_DIR}/cargo")
set(BORG_LIB "${CARGO_TARGET}/release/libborg_reborn_rs.a")
set(FEATURES "borgh")
if(BORG_FEATURES)
    set(FEATURES "borgh,${BORG_FEATURES}")
endif()
add_custom_target(rust_borg ALL
    COMMAND cargo build --release --locked -p borg-reborn-rs --features "${FEATURES}"
        --manifest-path "${REPO}/Cargo.toml" --target-dir "${CARGO_TARGET}"
    BYPRODUCTS "${BORG_LIB}"
    WORKING_DIRECTORY "${REPO}"
    USES_TERMINAL)
add_dependencies(angband rust_borg)
target_link_libraries(angband PRIVATE "${BORG_LIB}" pthread dl m)

# The game data, next to the executable, as upstream's build copies it.
add_custom_target(game_lib ALL
    COMMAND "${CMAKE_COMMAND}" -P "${TREE}/src/cmake/scripts/copy_with_exclude.cmake"
        -- "${CMAKE_BINARY_DIR}/game" "${TREE}/lib" "Makefile" ".deps" ".gitignore"
    VERBATIM)
