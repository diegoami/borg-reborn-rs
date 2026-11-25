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
#         [-DSUPPORT_GCU=ON] [-DBORG_RUST_TARGET=x86_64-pc-windows-gnu]
#   make -C build/rust
#
# harness/build.sh is the entry point (the zoo calls it).
cmake_minimum_required(VERSION 3.13)
project(borg-reborn-engine C)

set(ENGINE "${CMAKE_CURRENT_SOURCE_DIR}")
set(REPO "${ENGINE}/..")
set(TREE "${CMAKE_BINARY_DIR}/tree")

option(SUPPORT_X11 "Also build the X11 frontend (to watch the Borg play)" OFF)
# #823/#824: the curses frontend, to watch the Borg in a terminal (macOS,
# Windows' console, or a Linux terminal).
option(SUPPORT_GCU "Also build the curses (GCU) frontend (to watch the Borg in a terminal)" OFF)
# A cargo target triple when cross-compiling (e.g. x86_64-pc-windows-gnu from
# Linux with MinGW); empty: the host's.
set(BORG_RUST_TARGET "" CACHE STRING "cargo --target for the Rust Borg (cross builds only)")
option(SUPPORT_BORG_RECORDING "Enable the pinned Rust Angband recorder" OFF)
set(BORG_FEATURES "" CACHE STRING "Extra cargo features of the Rust Borg (tests only)")

# The patched copy of the tree, remade whenever the tree or a patch changes.
file(GLOB PATCHES "${ENGINE}/patches/*.patch")
# recorder-hooks.patch is applied during configuration too; a changed adapter
# must re-run CMake (and so re-apply it) before an incremental build.
set_property(DIRECTORY APPEND PROPERTY CMAKE_CONFIGURE_DEPENDS ${PATCHES} "${ENGINE}/UPSTREAM" "${ENGINE}/recorder-hooks.patch")
file(REMOVE_RECURSE "${TREE}")
# Windows (#823) cannot execute a shell script itself: MSYS2's bash runs it.
set(SHELL_PREFIX "")
if(WIN32)
    find_program(BORG_BASH bash REQUIRED)
    set(SHELL_PREFIX "${BORG_BASH}")
endif()
execute_process(COMMAND ${SHELL_PREFIX} "${ENGINE}/apply-patches.sh" "${TREE}" RESULT_VARIABLE PATCHED)
if(NOT PATCHED EQUAL 0)
    message(FATAL_ERROR "engine/apply-patches.sh failed")
endif()

if(SUPPORT_BORG_RECORDING)
    execute_process(COMMAND python3 "${REPO}/borg-reborn-rs/check-recorder-pin.py"
        "${REPO}/recorder-pin.json" "${REPO}/vendor/angband-recorder" RESULT_VARIABLE RECORDER_PIN)
    if(NOT RECORDER_PIN EQUAL 0)
        message(FATAL_ERROR "Recorder pin differs")
    endif()
    execute_process(COMMAND patch --batch --fuzz=0 -p1 --input "${ENGINE}/recorder-hooks.patch"
        WORKING_DIRECTORY "${TREE}" RESULT_VARIABLE RECORDER_PATCH)
    if(NOT RECORDER_PATCH EQUAL 0)
        message(FATAL_ERROR "Recorder hooks do not apply")
    endif()
endif()

include("${ENGINE}/sources.cmake")
list(TRANSFORM ANGBAND_CORE_SOURCES PREPEND "${TREE}/")

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/game")
add_compile_options(-Wall -Wextra -pedantic -Wno-unused-parameter)

# #823/#824: macOS and Windows packages carry no OpenSSL of their own; link
# libcrypto statically there (Linux links the system's libcrypto.so.3).
if(APPLE OR WIN32)
    set(OPENSSL_USE_STATIC_LIBS TRUE)
endif()
find_package(OpenSSL REQUIRED COMPONENTS Crypto)
set(FRONTENDS "${TREE}/src/main.c" "${ENGINE}/frontend/main-borgh.c" "${ENGINE}/frontend/synthetic-build.c")
if(SUPPORT_X11)
    list(APPEND FRONTENDS "${TREE}/src/main-x11.c")
endif()
if(SUPPORT_GCU)
    list(APPEND FRONTENDS "${TREE}/src/main-gcu.c")
endif()
add_executable(angband ${ANGBAND_CORE_SOURCES} ${FRONTENDS})
file(STRINGS "${ENGINE}/UPSTREAM" SYNTHETIC_PIN_LINE REGEX "^commit=")
string(REPLACE "commit=" "" SYNTHETIC_PIN "${SYNTHETIC_PIN_LINE}")
target_compile_definitions(angband PRIVATE SYNTHETIC_ENGINE_PIN="${SYNTHETIC_PIN}")
if(BORG_FEATURES MATCHES "(^|,)test-junk-consumables(,|$)")
    target_compile_definitions(angband PRIVATE BORGH_TEST_JUNK_CONSUMABLES)
endif()
set_target_properties(angband PROPERTIES C_STANDARD 99 C_EXTENSIONS ON)
target_include_directories(angband PRIVATE "${TREE}/src" "${ENGINE}/frontend")
# As upstream's self-contained build (SC_INSTALL): data in ./lib/ next to the
# executable. ALLOW_BORG as upstream's SUPPORT_BORG; BORG_HARNESS and
# USE_BORGH turn on our patch series and frontend.
target_compile_definitions(angband PRIVATE
    ALLOW_BORG BORG_HARNESS USE_BORGH
    DEFAULT_CONFIG_PATH="./lib/" DEFAULT_LIB_PATH="./lib/" DEFAULT_DATA_PATH="./lib/")
# Windows: upstream's main.c (not main-win.c) in console mode, as its own
# GCU and test frontends build it; borgh and gcu register through it.
if(WIN32)
    target_compile_definitions(angband PRIVATE WIN32_CONSOLE_MODE)
endif()
if(SUPPORT_GCU AND WIN32)
    # The static executable links MSYS2's static ncursesw (pkg-config
    # --static), not the DLL import library upstream's detection finds.
    find_package(PkgConfig REQUIRED)
    pkg_check_modules(NCURSESW REQUIRED ncursesw)
    target_compile_options(angband PRIVATE ${NCURSESW_STATIC_CFLAGS})
    target_compile_definitions(angband PRIVATE USE_GCU USE_NCURSES HAVE_USE_DEFAULT_COLORS
        NCURSES_STATIC MSYS2_ENCODING_WORKAROUND)
    # After the objects that use it (link order), with its own static deps.
    target_link_libraries(angband PRIVATE ${NCURSESW_STATIC_LDFLAGS})
elseif(SUPPORT_GCU)
    # Upstream's curses detection (ncursesw by pkg-config, then the system's
    # curses, as macOS's), from the patched tree.
    list(APPEND CMAKE_MODULE_PATH "${TREE}/src/cmake/modules")
    include("${TREE}/src/cmake/macros/GCU_Frontend.cmake")
    configure_gcu_frontend(angband)
endif()
if(SUPPORT_X11)
    find_package(X11 REQUIRED)
    target_compile_definitions(angband PRIVATE USE_X11)
    target_include_directories(angband PRIVATE ${X11_INCLUDE_DIR})
    target_link_libraries(angband PRIVATE ${X11_LIBRARIES})
endif()

# The Rust Borg: a staticlib built by cargo, with the harness feature.
set(CARGO_TARGET "${CMAKE_BINARY_DIR}/cargo")
set(BORG_LIB "${CARGO_TARGET}/release/libborg_reborn_rs.a")
set(CARGO_TARGET_ARGS "")
if(BORG_RUST_TARGET)
    set(BORG_LIB "${CARGO_TARGET}/${BORG_RUST_TARGET}/release/libborg_reborn_rs.a")
    set(CARGO_TARGET_ARGS --target "${BORG_RUST_TARGET}")
endif()
set(FEATURES "borgh")
if(SUPPORT_BORG_RECORDING)
    set(FEATURES "${FEATURES},recording")
endif()
if(BORG_FEATURES)
    set(FEATURES "${FEATURES},${BORG_FEATURES}")
endif()
add_custom_target(rust_borg ALL
    COMMAND cargo build --release --locked -p borg-reborn-rs --features "${FEATURES}"
        --manifest-path "${REPO}/Cargo.toml" --target-dir "${CARGO_TARGET}" ${CARGO_TARGET_ARGS}
    BYPRODUCTS "${BORG_LIB}"
    WORKING_DIRECTORY "${REPO}"
    USES_TERMINAL)
add_dependencies(angband rust_borg)
if(WIN32)
    # The Rust standard library's Windows imports, and a static executable
    # (no MinGW runtime DLLs next to it).
    target_link_libraries(angband PRIVATE "${BORG_LIB}" OpenSSL::Crypto
        ws2_32 crypt32 userenv bcrypt ntdll advapi32 synchronization m)
    target_link_options(angband PRIVATE -static)
else()
    target_link_libraries(angband PRIVATE "${BORG_LIB}" OpenSSL::Crypto pthread dl m)
endif()

# The game data, next to the executable, as upstream's build copies it.
add_custom_target(game_lib ALL
    COMMAND "${CMAKE_COMMAND}" -P "${TREE}/src/cmake/scripts/copy_with_exclude.cmake"
        -- "${CMAKE_BINARY_DIR}/game" "${TREE}/lib" "Makefile" ".deps" ".gitignore"
    VERBATIM)

if(SUPPORT_BORG_RECORDING)
    target_sources(angband PRIVATE "${REPO}/vendor/angband-recorder/host/angband.c"
        "${REPO}/vendor/angband-recorder/host/observations.c")
    target_include_directories(angband PRIVATE "${REPO}/vendor/angband-recorder/include"
        "${REPO}/vendor/angband-recorder/host")
    target_compile_definitions(angband PRIVATE BORG_RECORDING
        ANGBAND_RECORDER_BUILD="borg-reborn-recorder-v0.3.1")
    set_property(TARGET angband APPEND_STRING PROPERTY LINK_FLAGS " -Wl,--wrap=time")
endif()
