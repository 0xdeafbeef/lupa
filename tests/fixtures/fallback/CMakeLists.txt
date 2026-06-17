cmake_minimum_required(VERSION 3.10)

set(_PROJECT_NAME lupa-fixture)
set(_PROJECT_LANGUAGE CXX)
set(SUBPROJECT_LIST
    core
    cli
)

project(${_PROJECT_NAME} LANGUAGES ${_PROJECT_LANGUAGE})

execute_process(COMMAND node -e "console.log('fixture')"
    OUTPUT_VARIABLE NODE_OUTPUT)

foreach (SUBPROJ ${SUBPROJECT_LIST})
    add_subdirectory(${SUBPROJ})
endforeach ()
