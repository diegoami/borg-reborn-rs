# Recorder integration. The caller has already applied the versioned hook patch.
# For a Rust consumer set ANGBAND_RECORDER_RUST_LIBRARY to its existing Rust
# staticlib, built with the recorder dependency's ffi feature. Never link a
# second Rust staticlib into that executable.
set(_AR_ROOT "${CMAKE_CURRENT_LIST_DIR}/..")
function(angband_recorder CORE EXECUTABLE)
    if(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux" OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
        message(FATAL_ERROR "Recorder v0.1 ABI supports 64-bit Linux only")
    endif()
    set(ANGBAND_RECORDER_BUILD "angband-42cd156-recorder-0.1.0" CACHE STRING "Exact host build identity")
    target_sources(${CORE} PRIVATE "${_AR_ROOT}/host/angband.c" "${_AR_ROOT}/host/observations.c")
    target_include_directories(${CORE} PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/src" "${_AR_ROOT}/host" "${_AR_ROOT}/include")
    target_compile_definitions(${CORE} PUBLIC BORG_RECORDING
        ANGBAND_RECORDER_BUILD="${ANGBAND_RECORDER_BUILD}")
    target_include_directories(${EXECUTABLE} PRIVATE "${_AR_ROOT}/host" "${_AR_ROOT}/include")
    target_compile_definitions(${EXECUTABLE} PRIVATE BORG_RECORDING)
    if(NOT ANGBAND_RECORDER_RUST_LIBRARY)
        set(_AR_TARGET "${CMAKE_BINARY_DIR}/recorder-cargo")
        set(ANGBAND_RECORDER_RUST_LIBRARY "${_AR_TARGET}/release/libangband_recorder.a")
        add_custom_target(recorder_rust
            COMMAND cargo build --release --locked --features ffi
                --manifest-path "${_AR_ROOT}/Cargo.toml" --target-dir "${_AR_TARGET}"
            BYPRODUCTS "${ANGBAND_RECORDER_RUST_LIBRARY}"
            VERBATIM)
        add_dependencies(${EXECUTABLE} recorder_rust)
    endif()
    target_link_libraries(${EXECUTABLE} PRIVATE "${ANGBAND_RECORDER_RUST_LIBRARY}" pthread dl m)
    # Keep compatibility with CMake 3.7 supported by vanilla.
    set_property(TARGET ${EXECUTABLE} APPEND_STRING PROPERTY LINK_FLAGS " -Wl,--wrap=time")
endfunction()
