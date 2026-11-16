# Recorder integration. The caller has already applied the versioned hook patch.
# For a Rust consumer set ANGBAND_RECORDER_RUST_LIBRARY to its existing Rust
# staticlib, built with the recorder dependency's ffi feature. Never link a
# second Rust staticlib into that executable.
set(_AR_ROOT "${CMAKE_CURRENT_LIST_DIR}/..")
function(angband_recorder CORE EXECUTABLE)
    if(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux" OR NOT CMAKE_SIZEOF_VOID_P EQUAL 8)
        message(FATAL_ERROR "Recorder v0.1 ABI supports 64-bit Linux only")
    endif()
    # Unit executables reuse core objects in vanilla. Keep their core pristine
    # instead of injecting recorder references without the host/frontend ABI.
    get_target_property(_AR_CORE_SOURCES ${CORE} SOURCES)
    get_property(_AR_TARGETS DIRECTORY PROPERTY BUILDSYSTEM_TARGETS)
    set(_AR_UNIT_CORE "${CORE}RecorderOffTests")
    add_library(${_AR_UNIT_CORE} OBJECT ${_AR_CORE_SOURCES})
    set_target_properties(${_AR_UNIT_CORE} PROPERTIES EXCLUDE_FROM_ALL TRUE)
    foreach(_AR_PROP C_STANDARD C_EXTENSIONS INCLUDE_DIRECTORIES COMPILE_DEFINITIONS COMPILE_OPTIONS)
        get_target_property(_AR_VALUE ${CORE} ${_AR_PROP})
        if(_AR_VALUE)
            set_property(TARGET ${_AR_UNIT_CORE} PROPERTY ${_AR_PROP} "${_AR_VALUE}")
        endif()
    endforeach()
    foreach(_AR_TARGET ${_AR_TARGETS})
        get_target_property(_AR_KIND ${_AR_TARGET} TYPE)
        if(_AR_KIND STREQUAL "EXECUTABLE" AND NOT _AR_TARGET STREQUAL EXECUTABLE)
            get_target_property(_AR_SOURCES ${_AR_TARGET} SOURCES)
            string(REPLACE "$<TARGET_OBJECTS:${CORE}>" "$<TARGET_OBJECTS:${_AR_UNIT_CORE}>" _AR_REPLACED "${_AR_SOURCES}")
            set_property(TARGET ${_AR_TARGET} PROPERTY SOURCES "${_AR_REPLACED}")
        endif()
    endforeach()
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
